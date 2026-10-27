# Declaración de uso

> **Etapa temprana.** StarVektor está en desarrollo y su código es incompleto. Lo que sigue describe el diseño al que el proyecto se compromete, no un comportamiento ya implementado: hoy el programa no se engancha a ningún juego. Cualquier copia del código es parcial.

Este documento describe, en términos de **actos concretos**, qué hace StarVektor sobre los juegos y qué no hace. No es asesoría legal: es la descripción técnica que el proyecto se compromete a respetar. Si el comportamiento del programa cambia, este documento se actualiza antes que el código.

## Qué es

Software libre (licencia MIT prevista) que mejora la fluidez de presentación en monitores sin VRR generando fotogramas propios. Imita una función de la **pantalla** (la frecuencia variable), no una función de ningún juego.

## Alcance

- Juegos de **un solo jugador y sin conexión**.
- No se usa en juegos online ni en juegos con anticheat. En ellos StarVektor se apaga por defecto (lista de exclusión, hito H1.7 del [roadmap](roadmap.md)) y nunca se prueba en cuentas online.
- Las pruebas de comparación se hacen solo con juegos de un jugador, con las condiciones de [publisher-review.md](publisher-review.md).

## Qué hace

1. Observa la imagen que el sistema ya presenta, mediante las interfaces públicas del sistema (Vulkan, Wayland/X11 en Linux).
2. Genera fotogramas propios a partir de esa imagen.
3. Mide sus propios tiempos (tiempo de fotograma, costo por pasada, memoria de vídeo) y compara numéricamente el resultado con una referencia.
4. Procesa la imagen en memoria y la descarta. De los juegos comerciales solo se conservan **métricas numéricas**; ningún fotograma.

## Qué no hace

- No lee ni escribe la memoria del proceso del juego.
- No modifica ni parchea binarios, bibliotecas ni archivos del juego, y no lee sus archivos de datos.
- No extrae ni copia assets, modelos, texturas, audio ni texto.
- No intercepta ni emula tráfico de red ni protocolos del juego.
- No elude, desactiva ni interfiere con protecciones (DRM, anticheat, control de acceso).
- No automatiza el juego ni da ventaja de ningún tipo en partidas.
- No usa juegos comerciales para entrenar modelos (ver [data-policy.md](data-policy.md)).
- No usa la salida de otros generadores de fotogramas como objetivo de entrenamiento.
- No entrena ni ajusta ningún modelo con imágenes de juegos de terceros, tampoco en el equipo del usuario. Lo que se conserva son métricas numéricas y las marcas que el usuario hace a mano, sin imágenes.
- No tiene fin comercial: el software es gratuito.

## Niveles de integración

| Nivel (`integration_level`) | Qué usa | Sobre juegos de terceros |
|---|---|---|
| `image_only` | Solo la imagen presentada | Permitido, es el modo de caja negra |
| `upscaler` | Datos que el juego entrega a su escalador (profundidad, vectores de movimiento), interceptando la API pública | No se usa sin permiso del titular. Reservado para integración hecha por el propio desarrollador |
| `engine` | Datos del motor, mediante el SDK | Solo cuando el desarrollador del juego lo integra en su propio motor |

Cuando un desarrollador integra el SDK en su juego, StarVektor deja de ser software de terceros para ese juego y casi todas las restricciones sobre programas externos dejan de aplicar.

## Huella de cada enganche

La decisión entre capa Vulkan y gamescope (D2 en [design.md](design.md)) incluye medir la **huella**: si el código corre dentro del proceso del juego y a qué datos accede. Los resultados se registrarán aquí. Observación: gamescope también instala una capa Vulkan dentro del juego, así que "fuera del proceso" describe solo nuestro código.

| Enganche | Corre en el proceso del juego | Datos a los que accede | Estado |
|---|---|---|---|
| Capa Vulkan propia (modo solo observación) | Sí | Imagen presentada y tiempos de presentación | Por medir (H1.4) |
| Gamescope | Nuestro código, no | Imagen final del compositor | Por medir (H1.5) |

## Para estudios y editores

- StarVektor es gratuito y de código abierto. Un estudio puede integrarlo en su propio motor o usarlo internamente en desarrollo, sin coste.
- Si un titular de derechos nos pide dejar de usar su juego en las comparaciones o retirar una mención, se atiende.
- Para proponer una colaboración o un permiso de comparación, abre una propuesta en el repositorio.
