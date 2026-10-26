# Política de datos

El software aprende de **resultados**, no copia ni guarda contenido de juegos o SDK.

1. **Entrenar** solo con escenas propias (y datasets abiertos con licencia compatible).
2. **Juegos comerciales** (Control, Cyberpunk 2077, The Witcher 3 versión 2026, Crimson Desert): solo para evaluar calidad y costo, nunca para entrenar. Antes de capturar, revisar el EULA de cada uno.
3. **FSR 3, AFMF 2, DLSS FG, OptiScaler FG**: solo como referencia de comparación; su salida nunca entra al entrenamiento.
4. Toda secuencia debe tener una entrada en `data/sources.toml` con licencia, origen y fecha, y capturarse **sin** generación de fotogramas (`frame_generation = "off"`).
5. Un juego solo puede usarse para entrenar con permiso explícito (`permitted_game` + `permission`); los de la lista de solo evaluación quedan excluidos aunque haya permiso.

`scripts/check_sources.py` verifica lo que se puede comprobar de forma automática (campos, tipos, nombres de juegos y generadores conocidos). **No puede detectar** un juego comercial declarado como otra cosa: eso lo cubre la revisión humana al añadir cada fuente.
