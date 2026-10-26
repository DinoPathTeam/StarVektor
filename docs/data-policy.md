# Política de datos

StarVektor aprende de **resultados y de fotogramas de fuentes permitidas**; nunca guarda contenido, código ni binarios de juegos o SDK. No guardar no basta: los términos de uso pueden prohibir el acto de capturar y entrenar, así que **la fuente se decide antes de capturar**.

## Categorías

| Categoría | Qué entra | Uso |
|---|---|---|
| Permitida | Escenas propias renderizadas en motor abierto (Godot, Blender); datasets públicos con licencia abierta; juegos con permiso explícito | Entrenamiento y evaluación |
| Solo evaluación | Juegos comerciales (Control, Cyberpunk 2077, The Witcher 3 versión 2026, Crimson Desert) | Comparar calidad y costo; nunca entrenar |
| Referencia de comparación | Salidas de FSR 3, AFMF 2, DLSS FG y FG de OptiScaler | Medir si StarVektor lo hace mejor o peor; nunca como objetivo de entrenamiento |
| Prohibida | Código o binarios de SDK, assets extraídos de juegos, juegos cuyo EULA prohíba capturar o entrenar, capturas hechas con FG activo como entrada | Nunca |

Antes de capturar un juego comercial se revisa su EULA.

## Reglas del pipeline

1. La verdad de entrenamiento es el fotograma real a 120-240 fps (se descartan los intermedios), nunca la salida de un generador existente.
2. Cada secuencia lleva registro de procedencia: origen, versión, licencia o permiso, fecha, quién la capturó y hash del archivo.
3. La lista de fuentes está versionada en este repositorio (`data/sources.toml`); una secuencia sin entrada no entra al dataset.
4. Antes de publicar un modelo se hace una prueba de memorización: con entradas parecidas, no debe reproducir escenas de entrenamiento.
5. Los logs de telemetría no contienen imágenes ni datos del juego; el dataset de capturas es aparte y opcional.
6. El primer prototipo se entrena solo con escenas propias; los juegos comerciales se reservan para evaluar.

## Verificación automática

`scripts/check_sources.py` (también en CI) comprueba que cada fuente declare id, tipo, uso, licencia, origen, fecha y `frame_generation = "off"`; que los juegos con permiso lo indiquen; y que ningún juego de solo evaluación ni generador de referencia figure como entrenamiento.

**Límite:** el script no puede detectar un juego comercial declarado como otra cosa. Eso lo cubre la revisión humana al añadir cada fuente.

## Aportar datos

Si quieres aportar escenas, abre una propuesta indicando origen y licencia. Solo se aceptan fuentes que cumplan esta política.
