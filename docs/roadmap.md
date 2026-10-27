# Hoja de ruta

Siete fases en orden. No se pasa a la siguiente sin cumplir el criterio de salida. La regla que cruza todas: **StarVektor nunca debe dejar la imagen peor que no hacer nada.** Las decisiones de fondo están en [design.md](design.md).

Estado actual: **Fase 0 completada**; la Fase 1 es la siguiente.

## Fase 0. Cimientos (completada)

Objetivo: repositorio, capas, esquema del log, política de datos y TUI esqueleto con archivo TOML.

- [x] H0.1 Repositorio con capas separadas, compilando.
- [x] H0.2 Esquema del log versionado (`run`, `frame`, `event`, `summary`) y validador.
- [x] H0.3 Política de datos y lista de fuentes versionada, con verificación automática.
- [x] H0.4 TUI esqueleto que edita y guarda el archivo TOML.

**Salida:** un `run` válido escrito y verificado contra el esquema; la TUI edita y guarda la configuración. Se comprueba con `starvektor selftest`, que usa datos sintéticos.

## Fase 1. Línea base y prueba de enganche

Objetivo: medir sin generar nada y decidir cómo engancharse en Linux.

- [ ] H1.1 Medición sin generar nada: frame times, fps renderizados frente a presentados, VRAM.
- [ ] H1.2 Detección de la configuración de OptiScaler (solo escalado, FG, ambos).
- [ ] H1.3 Tres escenas fijas por juego de evaluación.
- [ ] H1.4 Prueba A: capa Vulkan implícita. Overhead, percentiles 1 % y 0.1 %, latencia, datos del motor disponibles, compatibilidad y huella (si corre en el proceso del juego y a qué datos accede).
- [ ] H1.5 Prueba B: gamescope, con las mismas métricas, incluida la huella.
- [ ] H1.6 Decisión D2 registrada con las cifras que la sustentan.
- [ ] H1.7 Lista de exclusión de juegos con anticheat: StarVektor apagado por defecto en ellos (D9).

**Salida:** repetir la misma escena da resultados comparables (tolerancia a definir con los datos) y D2 queda decidida.

## Fase 2. Núcleo clásico (nivel 1)

Objetivo: generación de fotogramas sin red.

- [ ] H2.1 Predictor de tiempos de fotograma.
- [ ] H2.2 Estimación de movimiento clásica y reproyección.
- [ ] H2.3 Protección del HUD por reglas y máscara de confianza heurística.
- [ ] H2.4 Regulador con histéresis y vigilancia de VRAM.
- [ ] H2.5 Modo «no intervenir» (nivel 0) probado.

**Salida:** costo dentro del presupuesto, percentil 1 % no peor que nativo y, como máximo, un fotograma de latencia añadida.

_Nota: el predictor (H2.1) y el regulador (H2.4) ya existen como código y pruebas unitarias en `sv-core`, pero no se dan por cumplidos hasta medirlos con un juego real._

## Fase 3. Datos y primer modelo (nivel 2)

Objetivo: dataset con procedencia y modelo v0 para máscara de errores y HUD.

- [ ] H3.1 Escenas propias en motor abierto capturadas a 120-240 fps, con registro de procedencia.
- [ ] H3.2 Dataset con los fotogramas descartados como verdad de entrenamiento.
- [ ] H3.3 Modelo mínimo para máscara de errores y protección del HUD (modelo v0).
- [ ] H3.4 Prueba de memorización superada.
- [ ] H3.5 Evaluación en escenas retenidas (SSIM/PSNR más revisión visual de HUD y bordes).

**Salida:** el modelo v0 mejora al nivel 1 y no memoriza escenas de entrenamiento.

## Fase 4. Comparación

Objetivo: medir contra el estado del arte y decidir si seguir.

- [ ] H4.1 Medición de FSR 3, AFMF 2 y FG de OptiScaler en los juegos de evaluación.
- [ ] H4.2 Tabla comparativa de calidad, latencia y ms, en las cuatro configuraciones por escena.
- [ ] H4.3 Decisión explícita: seguir, cambiar de enfoque o recortar alcance.

**Salida:** decisión registrada en este repositorio, con los datos.

## Fase 5. Red completa (nivel 3)

Objetivo: estimador de movimiento aprendido, relleno de zonas descubiertas, perfiles por juego y familia de modelos.

- [ ] H5.1 Estimador de movimiento aprendido.
- [ ] H5.2 Relleno de zonas descubiertas (inpainting).
- [ ] H5.3 Perfiles por juego.
- [ ] H5.4 Familia de modelos (grande, mediano, mínimo) por destilación.
- [ ] H5.5 Selección automática de modelo por presupuesto de ms y VRAM.

**Salida:** la selección automática funciona y respeta el tope de VRAM en una GPU de 8 GB.

## Fase 6. Portabilidad y hardware antiguo

Objetivo: capa de plataforma para Windows y degradación escalonada en GPUs sin aceleración de IA.

- [ ] H6.1 Capa de plataforma para Windows.
- [ ] H6.2 Pruebas en GPUs antiguas (RX 580, GTX 1650) con degradación escalonada.
- [ ] H6.3 Revisión de la decisión sobre GUI (D3).

**Salida:** el mismo núcleo corre en ambos sistemas; en hardware sin aceleración de IA, nivel 1 o 2 sin empeorar la imagen.
