# Control de la máquina

El Control de la máquina es el lugar principal para conectarse al controlador y ejecutar tareas.

## Conexión
1. Seleccione el puerto serie.
2. Haga clic en **Conectar**.
3. Consulte la **Consola del dispositivo** para ver los mensajes de inicio.

Si tiene problemas de conexión:
- Verifique los permisos del dispositivo serie (p. ej., `/dev/ttyACM0`).
- Confirme la velocidad de transmisión correcta para su firmware.

## Movimientos en manual
- Utilice el panel de control en pantalla.
- Utilice el control manual con teclado (si está habilitado) para un posicionamiento rápido.
- Configure **Paso (mm)** y **Avance manual** para controlar el movimiento.

## Inicio / Desbloqueo / Reinicio
- **Inicio** ejecuta el ciclo de inicio del firmware (requiere `$22=1` en GRBL).
- **Desbloqueo** borra las alarmas si el controlador está en estado de alarma.
- **Reinicio** realiza un reinicio suave.

## Transmisión de un trabajo
1. Cargue o genere el G-code.
2. Haga clic en **Enviar** para iniciar la transmisión.
3. Use **Pausa/Reanudar** según sea necesario.
4. **Detener** cancela la transmisión.

## Relacionado
- [Consola del dispositivo](help:device_console)
- [Visualizador](help:visualizer)
- [Índice](help:index)
