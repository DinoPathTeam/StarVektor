# Diseño y hoja de prototipo

Documento técnico del prototipo. **Todas las cifras son estimaciones de diseño, no mediciones**: se validarán con consultas de tiempo (timestamp queries) en la GPU real, con el juego corriendo, a partir de la Fase 1.

## Decisiones vigentes

Si una decisión cambia, se actualiza aquí con fecha y motivo. Un hito no se da por cumplido sin su criterio de salida.

| | Decisión |
|---|---|
| D1. Lenguaje | El núcleo (predictor, regulador, máscara de confianza, telemetría, TUI) es Rust. C++ solo donde el ecosistema lo imponga (un shim fino de la capa Vulkan o SDK de inferencia del fabricante), con frontera estrecha: interfaz C pequeña, llamadas gruesas por fotograma, reglas claras de propiedad de búferes y dependencia en un solo sentido. Los shaders se compilan aparte. |
| D2. Enganche en Linux | Se decide por medición, no por preferencia: capa Vulkan implícita frente a gamescope, con el mismo protocolo. Gana la de menor overhead y mejor ritmo de presentación; la otra se conserva solo si aporta algo que la primera no tiene. Se añade un criterio de **huella**: qué toca cada opción (si corre dentro del proceso del juego y a qué datos accede), porque alimenta la [declaración de uso](usage-statement.md). Nota: gamescope también instala una capa Vulkan dentro del juego, así que "fuera del proceso" solo describe nuestro código, no todo el camino. Hipótesis a verificar: un híbrido (la capa extrae datos del motor y gamescope presenta a tiempo), que además duplicaría la huella. **Pendiente (Fase 1).** |
| D3. Configuración | TUI sobre un archivo TOML y variables de entorno, perfiles por juego y un overlay mínimo (frame time, nivel activo, atajo para marcar artefactos). Sin GUI por ahora. |
| D4. Datos | Entrenamiento solo con escenas propias en motor abierto y fuentes con permiso explícito. Juegos comerciales solo para evaluar. Salidas de otros generadores, solo como referencia. Procedencia obligatoria. Ver [data-policy.md](data-policy.md). |
| D5. Regla transversal | Nunca dejar la imagen peor que no hacer nada. Ante la duda, repetir el fotograma o apagarse (nivel 0). |
| D6. Orden de trabajo | Linux primero; Vulkan como base para que la capa de plataforma sea lo único que cambie al portar. Hardware antiguo después de que funcione en el moderno. |
| D8. Huella mínima (caja negra) | StarVektor solo observa la imagen que el sistema ya presenta (Vulkan, Wayland/X11) y mide sus propios tiempos. No lee ni escribe memoria del juego, no modifica binarios ni archivos, no extrae assets, no intercepta red y no interfiere con protecciones. El nivel que usa datos del escalador del juego (profundidad, vectores) queda reservado para integración hecha por el propio desarrollador, y no se usa sobre juegos de terceros sin permiso. Detalle en [usage-statement.md](usage-statement.md). |
| D9. Alcance | Juegos de un solo jugador y sin conexión. Los juegos con anticheat quedan fuera: StarVektor se apaga por defecto en ellos (lista de exclusión, hito H1.7) y no se prueba en cuentas online. |
| D10. Licencia y distribución | Prevista: MIT, para que las distros puedan incluirlo como dependencia y los motores como SDK. Los pesos de modelos y los datasets se licenciarán aparte. Todavía no hay archivo `LICENSE`. |
| D7. Cadena con OptiScaler | No apilar dos generadores de fotogramas. Detectar la configuración en vez de asumirla; medir siempre fps renderizados y presentados. |

## Capas

1. **Núcleo agnóstico:** predicción de tiempos, regulador (con histéresis), máscara de confianza, detección de configuración, telemetría. Sin llamadas al sistema operativo.
2. **Backend gráfico:** Vulkan compute detrás de una interfaz pequeña. Inferencia con interfaz abstracta (unidades de IA hoy, shaders genéricos después).
3. **Capa de plataforma:** captura y presentación (Wayland/gamescope en Linux), lectura de entrada, enganche con OptiScaler y el escalador del juego. Es lo único que cambia entre Linux y Windows.

## Configuración: TUI sí, GUI por ahora no

Una TUI es adecuada para una dependencia: corre por SSH o en una terminal, no necesita servidor gráfico propio y se mantiene con poco código. Para que no sea el único camino, toda opción vive en el archivo TOML y en variables de entorno (sirve en una opción de lanzamiento de Steam o en un script), hay perfiles por juego y un overlay mínimo para probar. Se añadiría una GUI delgada sobre el mismo archivo si se apunta a usuarios no técnicos, sobre todo en Windows.

## Presupuesto de tiempo (1440p, estimado)

Regla base: el costo por fotograma debe ser el 10 a 15 % del tiempo base T. Con 60 fps base (T ≈ 16.7 ms), unos 1.7 a 2.5 ms.

| Pasada | ms objetivo |
|---|---|
| Captura sin copia | 0.3 |
| Estimación de movimiento | 0.8 a 1.0 |
| Reproyección y mezcla | 0.4 a 0.5 |
| Máscara de confianza y protección del HUD | 0.3 a 0.5 |
| Presentación | 0.2 |
| **Total** | **≈ 2.0 a 2.5** |

## Presupuesto de VRAM (8 GB, estimado)

| Elemento | MB |
|---|---|
| Historial de 2 a 3 fotogramas | 30 a 45 |
| Vectores y profundidad aproximada (media resolución) | 10 a 20 |
| Máscaras | < 5 |
| Pesos del modelo | 2 a 10 |
| Activaciones (media resolución) | ≈ 30 |
| **Objetivo / tope duro** | **150 a 250 / 400** |

Si la VRAM libre baja de unos 400 MB (`VK_EXT_memory_budget`), el regulador baja un nivel. Por eso importa la VRAM por modelo: el tamaño del modelo se elige según el presupuesto disponible.

## Casos de prueba

Tres escenas fijas por juego (exploración, combate con partículas, iluminación cambiante) en cuatro configuraciones: nativo, con escalado, con escalado + FG de OptiScaler y con StarVektor. Los juegos comerciales se usan solo para evaluar.

| Juego | Qué pone a prueba |
|---|---|
| Cyberpunk 2077 | HUD denso, neones, transparencias |
| Control | Reflejos, partículas, geometría destructible |
| The Witcher 3 (versión 2026) | Vegetación densa, cielos, shimmering |
| Crimson Desert | Mapa enorme, vegetación, clima variado, destrucción, iluminación dinámica |

Para la fase de hardware antiguo se añadirán juegos online competitivos, donde se espera que StarVektor quede en nivel 0.

## Decisiones abiertas

- Capa Vulkan implícita o dentro de gamescope (Fase 1, con mediciones).
- Orden de la cadena cuando el usuario activa la generación de fotogramas de OptiScaler: se mide en la Fase 1; la recomendación de partida es no apilar dos generadores.
