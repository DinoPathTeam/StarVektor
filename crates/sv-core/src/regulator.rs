//! Regulador de niveles (hito H2.4, versión inicial).
//!
//! Decide el nivel de StarVektor a partir de cuatro números por fotograma:
//! el tiempo base del juego, el periodo de refresco del monitor, lo que cuesta
//! StarVektor y la VRAM libre. No sabe nada de GPU, monitores ni ficheros.
//!
//! Reglas:
//! - **Bajar es rápido** (`panic_frames` muestras malas seguidas): si el costo
//!   supera el presupuesto o la VRAM libre cae bajo el umbral.
//! - **Subir es lento** (`settle_frames` muestras buenas seguidas): hace falta
//!   holgura de sobra. La histéresis evita que oscile.
//! - **Apagado si no hace falta**: si el juego ya iguala el refresco, nivel 0.
//! - **Retroceso exponencial**: cada bajada forzada duplica la espera para
//!   volver a subir (hasta `max_backoff`); así un nivel que no cabe no parpadea.
//!
//! Todos los umbrales son hipótesis iniciales (ver la hoja de prototipo): se
//! ajustan con las mediciones de la fase 1.

use sv_types::Level;

/// Parámetros del regulador.
#[derive(Debug, Clone, PartialEq)]
pub struct RegulatorConfig {
    /// Nivel máximo permitido (tope puesto por el usuario o el perfil).
    pub max_level: Level,
    /// Fracción del tiempo base que StarVektor puede gastar (0.10 a 0.15).
    pub budget_fraction: f64,
    /// VRAM libre mínima, en MB. Por debajo se baja de nivel.
    pub vram_floor_mb: u32,
    /// VRAM libre extra sobre el mínimo que hace falta para subir, en MB.
    pub vram_margin_mb: u32,
    /// Para subir, el costo debe ser como mucho `budget * up_margin`.
    pub up_margin: f64,
    /// Muestras malas seguidas antes de bajar.
    pub panic_frames: u32,
    /// Muestras buenas seguidas antes de subir.
    pub settle_frames: u32,
    /// Multiplicador máximo de la espera tras bajadas forzadas.
    pub max_backoff: u32,
    /// Tolerancia al comparar el tiempo base con el refresco.
    pub refresh_tolerance: f64,
}

impl Default for RegulatorConfig {
    fn default() -> Self {
        Self {
            max_level: Level::FullNet,
            budget_fraction: 0.12,
            vram_floor_mb: 400,
            vram_margin_mb: 200,
            up_margin: 0.6,
            panic_frames: 3,
            settle_frames: 120,
            max_backoff: 8,
            refresh_tolerance: 0.02,
        }
    }
}

/// Medidas de un fotograma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Tiempo base del juego (sin StarVektor), en ms.
    pub base_frame_ms: f64,
    /// Periodo de refresco del monitor, en ms.
    pub refresh_ms: f64,
    /// Costo de StarVektor en este fotograma, en ms (0 si está apagado).
    pub overhead_ms: f64,
    /// VRAM libre, en MB.
    pub vram_free_mb: u32,
}

/// Motivo de un cambio de nivel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// El costo superó el presupuesto.
    OverBudget,
    /// La VRAM libre cayó bajo el umbral.
    VramLow,
    /// El juego ya iguala el refresco: no hace falta intervenir.
    NotNeeded,
    /// El juego no llega al refresco: conviene intervenir.
    Needed,
    /// Hay holgura de tiempo y de memoria para un nivel más alto.
    Headroom,
    /// El nivel actual superaba el tope configurado.
    Capped,
}

impl Reason {
    /// Texto estable para el log (`event.detail.reason`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::OverBudget => "over_budget",
            Reason::VramLow => "vram_low",
            Reason::NotNeeded => "not_needed",
            Reason::Needed => "needed",
            Reason::Headroom => "headroom",
            Reason::Capped => "capped",
        }
    }
}

/// Cambio de nivel decidido por el regulador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub from: Level,
    pub to: Level,
    pub reason: Reason,
}

/// Máquina de estados del regulador.
#[derive(Debug, Clone)]
pub struct Regulator {
    cfg: RegulatorConfig,
    level: Level,
    bad: u32,
    climb: u32,
    idle: u32,
    needed: u32,
    backoff: u32,
    since_change: u64,
}

impl Regulator {
    /// Empieza apagado (nivel 0): primero se comprueba que hace falta.
    #[must_use]
    pub fn new(cfg: RegulatorConfig) -> Self {
        Self {
            cfg,
            level: Level::Off,
            bad: 0,
            climb: 0,
            idle: 0,
            needed: 0,
            backoff: 1,
            since_change: 0,
        }
    }

    #[must_use]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Multiplicador de espera vigente (1 = sin retroceso).
    #[must_use]
    pub fn backoff(&self) -> u32 {
        self.backoff
    }

    /// Cambia los parámetros en caliente (por ejemplo, al cambiar de perfil).
    pub fn set_config(&mut self, cfg: RegulatorConfig) {
        self.cfg = cfg;
    }

    /// Procesa un fotograma. Devuelve el cambio de nivel, si lo hay.
    pub fn observe(&mut self, s: &Sample) -> Option<Transition> {
        self.since_change = self.since_change.saturating_add(1);

        // Una racha larga y estable en un nivel activo borra el retroceso.
        if self.level != Level::Off
            && self.backoff > 1
            && self.since_change >= u64::from(self.cfg.settle_frames) * 8
        {
            self.backoff = 1;
        }

        if self.level > self.cfg.max_level {
            return Some(self.go(self.cfg.max_level, Reason::Capped));
        }

        let needs_help = s.base_frame_ms > s.refresh_ms * (1.0 + self.cfg.refresh_tolerance);
        let vram_ok = s.vram_free_mb >= self.cfg.vram_floor_mb;
        let settle = self.cfg.settle_frames.saturating_mul(self.backoff);

        if self.level == Level::Off {
            if self.cfg.max_level > Level::Off && needs_help && vram_ok {
                self.needed += 1;
            } else {
                self.needed = 0;
            }
            if self.needed >= settle {
                return Some(self.go(Level::Classic, Reason::Needed));
            }
            return None;
        }

        let budget = s.base_frame_ms * self.cfg.budget_fraction;
        if s.overhead_ms > budget || !vram_ok {
            self.bad += 1;
            self.climb = 0;
            self.idle = 0;
        } else {
            self.bad = 0;
        }

        if self.bad >= self.cfg.panic_frames {
            let reason = if vram_ok {
                Reason::OverBudget
            } else {
                Reason::VramLow
            };
            self.backoff = self
                .backoff
                .saturating_mul(2)
                .min(self.cfg.max_backoff.max(1));
            return Some(self.go(self.level.down(), reason));
        }
        if self.bad > 0 {
            return None;
        }

        if needs_help {
            self.idle = 0;
        } else {
            self.idle += 1;
            if self.idle >= self.cfg.settle_frames {
                return Some(self.go(Level::Off, Reason::NotNeeded));
            }
        }

        let headroom = s.overhead_ms <= budget * self.cfg.up_margin
            && s.vram_free_mb
                >= self
                    .cfg
                    .vram_floor_mb
                    .saturating_add(self.cfg.vram_margin_mb);
        if needs_help && headroom && self.level < self.cfg.max_level {
            self.climb += 1;
        } else {
            self.climb = 0;
        }
        if self.climb >= settle {
            return Some(self.go(self.level.up(), Reason::Headroom));
        }
        None
    }

    fn go(&mut self, to: Level, reason: Reason) -> Transition {
        let from = self.level;
        self.level = to;
        self.bad = 0;
        self.climb = 0;
        self.idle = 0;
        self.needed = 0;
        self.since_change = 0;
        Transition { from, to, reason }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFRESH: f64 = 1000.0 / 60.0;

    fn cfg() -> RegulatorConfig {
        RegulatorConfig {
            settle_frames: 5,
            panic_frames: 3,
            ..RegulatorConfig::default()
        }
    }

    /// Juego a ~45 fps (necesita ayuda en un monitor de 60 Hz).
    fn slow(overhead_ms: f64, vram_free_mb: u32) -> Sample {
        Sample {
            base_frame_ms: 22.0,
            refresh_ms: REFRESH,
            overhead_ms,
            vram_free_mb,
        }
    }

    /// Juego a 100 fps (no necesita ayuda en 60 Hz).
    fn fast(overhead_ms: f64) -> Sample {
        Sample {
            base_frame_ms: 10.0,
            refresh_ms: REFRESH,
            overhead_ms,
            vram_free_mb: 4000,
        }
    }

    fn feed(r: &mut Regulator, s: &Sample, n: u32) -> Option<Transition> {
        let mut last = None;
        for _ in 0..n {
            if let Some(t) = r.observe(s) {
                last = Some(t);
            }
        }
        last
    }

    /// Alimenta fotogramas lentos hasta que pasa de apagado a nivel 1.
    fn wake(r: &mut Regulator) {
        for _ in 0..1000 {
            if let Some(t) = r.observe(&slow(0.0, 4000)) {
                assert_eq!((t.from, t.to), (Level::Off, Level::Classic));
                return;
            }
        }
        panic!("el regulador no despertó");
    }

    #[test]
    fn starts_off_and_wakes_when_the_game_is_slower_than_refresh() {
        let mut r = Regulator::new(cfg());
        assert_eq!(r.level(), Level::Off);
        assert_eq!(feed(&mut r, &slow(0.0, 4000), 4), None);
        let t = r.observe(&slow(0.0, 4000)).unwrap();
        assert_eq!(
            t,
            Transition {
                from: Level::Off,
                to: Level::Classic,
                reason: Reason::Needed
            }
        );
    }

    #[test]
    fn stays_off_when_the_game_already_matches_refresh() {
        let mut r = Regulator::new(cfg());
        assert_eq!(feed(&mut r, &fast(0.0), 500), None);
        assert_eq!(r.level(), Level::Off);
    }

    #[test]
    fn does_not_wake_without_vram() {
        let mut r = Regulator::new(cfg());
        assert_eq!(feed(&mut r, &slow(0.0, 100), 500), None);
    }

    #[test]
    fn drops_fast_when_over_budget() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        // presupuesto = 22 * 0.12 = 2.64 ms
        assert_eq!(feed(&mut r, &slow(5.0, 4000), 2), None);
        let t = r.observe(&slow(5.0, 4000)).unwrap();
        assert_eq!(
            t,
            Transition {
                from: Level::Classic,
                to: Level::Off,
                reason: Reason::OverBudget
            }
        );
    }

    #[test]
    fn a_transient_spike_does_not_drop() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        for _ in 0..20 {
            assert_eq!(feed(&mut r, &slow(5.0, 4000), 2), None);
            assert_eq!(r.observe(&slow(2.0, 4000)), None);
        }
        assert_eq!(r.level(), Level::Classic);
    }

    #[test]
    fn drops_when_vram_runs_low() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        let t = feed(&mut r, &slow(1.0, 100), 3).unwrap();
        assert_eq!(t.reason, Reason::VramLow);
        assert_eq!(t.to, Level::Off);
    }

    #[test]
    fn climbs_slowly_with_headroom_up_to_the_cap() {
        let mut r = Regulator::new(RegulatorConfig {
            max_level: Level::MiniNet,
            ..cfg()
        });
        wake(&mut r);
        // 1.0 ms <= 2.64 * 0.6 = 1.58 ms
        assert_eq!(feed(&mut r, &slow(1.0, 4000), 4), None);
        let t = r.observe(&slow(1.0, 4000)).unwrap();
        assert_eq!(
            t,
            Transition {
                from: Level::Classic,
                to: Level::MiniNet,
                reason: Reason::Headroom
            }
        );
        assert_eq!(
            feed(&mut r, &slow(1.0, 4000), 100),
            None,
            "no pasa del tope"
        );
        assert_eq!(r.level(), Level::MiniNet);
    }

    #[test]
    fn does_not_climb_without_enough_headroom() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        // Dentro del presupuesto (2.0 < 2.64) pero sin holgura (2.0 > 1.58).
        assert_eq!(feed(&mut r, &slow(2.0, 4000), 500), None);
        assert_eq!(r.level(), Level::Classic);
    }

    #[test]
    fn turns_off_when_no_longer_needed() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        assert_eq!(feed(&mut r, &fast(0.5), 4), None);
        let t = r.observe(&fast(0.5)).unwrap();
        assert_eq!(
            t,
            Transition {
                from: Level::Classic,
                to: Level::Off,
                reason: Reason::NotNeeded
            }
        );
        assert_eq!(r.backoff(), 1, "apagarse por no hacer falta no castiga");
    }

    #[test]
    fn backoff_doubles_the_wait_after_each_forced_drop() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        feed(&mut r, &slow(5.0, 4000), 3); // bajada forzada -> backoff 2
        assert_eq!(r.backoff(), 2);
        assert_eq!(feed(&mut r, &slow(0.0, 4000), 9), None, "espera 5 * 2 = 10");
        assert!(r.observe(&slow(0.0, 4000)).is_some());
        feed(&mut r, &slow(5.0, 4000), 3); // segunda bajada -> backoff 4
        assert_eq!(r.backoff(), 4);
        assert_eq!(
            feed(&mut r, &slow(0.0, 4000), 19),
            None,
            "espera 5 * 4 = 20"
        );
        assert!(r.observe(&slow(0.0, 4000)).is_some());
    }

    #[test]
    fn backoff_is_capped() {
        let mut r = Regulator::new(RegulatorConfig {
            max_backoff: 4,
            ..cfg()
        });
        for _ in 0..6 {
            wake(&mut r);
            feed(&mut r, &slow(5.0, 4000), 3);
        }
        assert_eq!(r.backoff(), 4);
    }

    #[test]
    fn backoff_resets_after_a_long_stable_run() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        feed(&mut r, &slow(5.0, 4000), 3);
        assert_eq!(r.backoff(), 2);
        wake(&mut r);
        // Estable: dentro del presupuesto, sin holgura para subir. 5 * 8 = 40 muestras.
        assert_eq!(feed(&mut r, &slow(2.0, 4000), 60), None);
        assert_eq!(r.backoff(), 1);
    }

    #[test]
    fn lowering_the_cap_forces_the_level_down() {
        let mut r = Regulator::new(cfg());
        wake(&mut r);
        r.set_config(RegulatorConfig {
            max_level: Level::Off,
            ..cfg()
        });
        let t = r.observe(&slow(1.0, 4000)).unwrap();
        assert_eq!(
            t,
            Transition {
                from: Level::Classic,
                to: Level::Off,
                reason: Reason::Capped
            }
        );
    }

    #[test]
    fn a_zero_cap_never_leaves_off() {
        let mut r = Regulator::new(RegulatorConfig {
            max_level: Level::Off,
            ..cfg()
        });
        assert_eq!(feed(&mut r, &slow(0.0, 4000), 1000), None);
        assert_eq!(r.level(), Level::Off);
    }

    #[test]
    fn reason_strings_are_stable() {
        assert_eq!(Reason::OverBudget.as_str(), "over_budget");
        assert_eq!(Reason::VramLow.as_str(), "vram_low");
        assert_eq!(Reason::NotNeeded.as_str(), "not_needed");
        assert_eq!(Reason::Needed.as_str(), "needed");
        assert_eq!(Reason::Headroom.as_str(), "headroom");
        assert_eq!(Reason::Capped.as_str(), "capped");
    }
}
