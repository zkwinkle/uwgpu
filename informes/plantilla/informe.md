---
toc: yes
header-includes: yes
geometry: "left=1.6cm,right=1.6cm,top=1.6cm,bottom=1.6cm"
gradient: no
underline: no
colorlinks: yes
titlepage: no
papersize: legal

title: Informe de Avance
subtitle: µwgpu
author: Ignacio Vargas Campos
id: 2019053776
date: II Semestre 2024
institute: Instituto Tecnológico de Costa Rica
department: Escuela de Ingeniería en Computadores
---

# 1. Datos del documento

# 1.1. Nombre del estudiante

Ignacio Vargas Campos

## 1.2. Fecha y número del informe

- Fecha: [TODO: Insertar fecha]
- Número: Informe de avance \#[TODO: Insertar número de informe]

## 1.3. Centro de investigación o empresa donde se desarrolla el proyecto

El proyecto se está llevando a cabo como parte de un proyecto de investigación en el Instituto Tecnológico de Costa Rica.

# 2. Actividades realizadas en el periodo reportado

[TODO: Descripción de actividades realizadas. Prosa.]

Posibles temas:
- ¿Qué herramientas usó?
- ¿Qué dificultades ha tenido?
- ¿Les dedicó más o menos tiempo del presupuestado?
- ¿Ha necesitado ayuda?
- ¿El proyecto va adelantado o retrasado con respecto al cronograma?
- ¿Por qué no se trabajó en las actividades planeadas para el período reportado según su plan de proyecto?
- ¿Ha tenido que agregar tareas imprevistas?
- ¿Ha tenido que eliminar tareas planeadas que no fueron requeridas?

# 3. Dificultades encontradas

[TODO: Breve descripción de dificultades y si fueron resueltas.]

# 4. Cambios en el alcance y/o actividades

## Actividades eliminadas

- Actividad 1
- Actividad 2
- [TODO: Poner actividades que hayan sido eliminadas o eliminar esta sección]

## Actividades agregadas

- Actividad 1
- Actividad 2
- [TODO: Poner actividades que hayan sido agregadas o eliminar esta sección]

## Cambios en el orden

[TODO: Poner cambios en el orden de las actividades o eliminar esta sección]

| Actividad | Orden original | Nuevo orden | Mótivo |
|-----------|----------------|-------------|--------|
| Actividad X | Semana X, después de actividad B y antes de actividad F | Semana Y, después de actividad W y antes de actividad Z | Mótivo |

### Cronograma actualizado

| Semana | Actividades a realizar | Horas estimadas de trabajo por semana |
| ------ | ---------------------- | ------------------------------------- |
| 1      | 100                    | 10                                    |
| 2      | 200                    | 10                                    |
| 3      | 300                    | 15                                    |
| 4      | 400                    | 8                                     |
| 5      | 500                    | 8                                     |
| 6      | 600, 601, 602          | 11                                    |
| 7      | 700                    | 8                                     |
| 8      | 800                    | 12                                    |
| 9      | 900, 901               | 8                                     |
| 10     | 1000, 1001             | 10                                    |
| 11     | 1100, 1101, 1102       | 12                                    |
| 12     | 1200, 1201             | 10                                    |
| 13     | 1300, 1301             | 10                                    |
| 14     | 1400, 1401             | 10                                    |
| 15     | 1500, 1501             | 12                                    |
| 16     | 1600, 1601             | 14                                    |

\blscape

# 5. Análisis de valor ganado

[TODO: Llenar columnas con datos reales para todas las actividades. Crear fila de totales al final.]

| ID        | Actividad                                                | Presupuesto (horas) | % Valor Planeado | PV (horas) | AC (horas) | % trabajo Completado | EV (horas) | CPI (horas) | SPI (horas) | Fecha inicio planeada | Finalización planeada | Fecha inicio real | Finalización real |
| --------- | -------------------------------------------------------- | ------------------- | ---------------- | ---------- | ---------- | -------------------- | ---------- | ----------- | ----------- | --------------------- | --------------------- | ----------------- | ----------------- |
| 100           | Experimentar y familiarizar con wgpu                                                                                                                                                  | 10                  | 0%             | 10         |            | 0%                 |            |             |             | 7/22/2024             | 7/26/2024             |                   |                   |
| 200           | Definir requisitos                                                                                                                                                                    | 10                  | 0%             | 10         |            | 0%                 |            |             |             | 7/29/2024             | 8/2/2024              |                   |                   |
| 300           | Crear setup inicial para pruebas con pipeline mínimo y una prueba de multiplicación matricial                                                                                         | 15                  | 0%             | 15         |            | 0%                 |            |             |             | 8/5/2024              | 8/9/2024              |                   |                   |
| 400           | Agregar una prueba de ancho de banda de memoria (la de copias entre buffers) para ya tener una prueba de los 2 tipos principales que hay y tener una mejor idea de qué es necesario   | 8                   | 0%             | 8          |            | 0%                 |            |             |             | 8/12/2024             | 8/16/2024             |                   |                   |
| 500           | Plan de proyecto                                                                                                                                                                      | 8                   | 0%             | 8          |            | 0%                 |            |             |             | 8/19/2024             | 8/23/2024             |                   |                   |
| 600           | Diseñar API y arquitectura de biblioteca de framework para crear microbenchmarks                                                                                                      | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 8/26/2024             | 8/28/2024             |                   |                   |
| 601           | Diseñar estructura de página web                                                                                                                                                      | 2                   | 0%             | 2          |            | 0%                 |            |             |             | 8/28/2024             | 8/28/2024             |                   |                   |
| 602           | Diseñar arquitectura de servidor que sirve página web y comunica con base de datos                                                                                                    | 4                   | 0%             | 4          |            | 0%                 |            |             |             | 8/29/2024             | 8/30/2024             |                   |                   |
| 700           | Redactar el documento de diseño formal                                                                                                                                                | 8                   | 0%             | 8          |            | 0%                 |            |             |             | 9/2/2024              | 9/6/2024              |                   |                   |
| 800           | Implementar API y arquitectura de biblioteca, reescribiendo las pruebas existentes para adaptarse a la biblioteca ya definida                                                         | 12                  | 0%             | 12         |            | 0%                 |            |             |             | 9/9/2024              | 9/13/2024             |                   |                   |
| 900           | Escribir microbenchmarks existentes como una biblioteca separada                                                                                                                      | 4                   | 0%             | 4          |            | 0%                 |            |             |             | 9/16/2024             | 9/18/2024             |                   |                   |
| 901           | Implementar interfaz CLI como wrapper de la biblioteca de microbenchmarks                                                                                                             | 4                   | 0%             | 4          |            | 0%                 |            |             |             | 9/19/2024             | 9/20/2024             |                   |                   |
| 1000          | Implementar servidor que sirve página web con configuración local                                                                                                                     | 8                   | 0%             | 8          |            | 0%                 |            |             |             | 9/23/2024             | 9/25/2024             |                   |                   |
| 1001          | Agregar detalles de producción al servidor (como configuración de DB real)                                                                                                            | 2                   | 0%             | 2          |            | 0%                 |            |             |             | 9/25/2024             | 9/27/2024             |                   |                   |
| 1100          | Crear el archivo de nix para empaquetar el servidor                                                                                                                                   | 3                   | 0%             | 3          |            | 0%                 |            |             |             | 9/30/2024             | 10/1/2024             |                   |                   |
| 1101          | Poner el servidor en el servicio de hosting                                                                                                                                           | 4                   | 0%             | 4          |            | 0%                 |            |             |             | 10/1/2024             | 10/2/2024             |                   |                   |
| 1102          | Implementar microbenchmark de reducción                                                                                                                                               | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/3/2024             | 10/4/2024             |                   |                   |
| 1200          | Implementar microbenchmark de convolución                                                                                                                                             | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/7/2024             | 10/9/2024             |                   |                   |
| 1201          | Implementar microbenchmark de scan                                                                                                                                                    | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/9/2024             | 10/11/2024            |                   |                   |
| 1300          | Implementar microbenchmark de accesos de memoria secuenciales                                                                                                                         | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/14/2024            | 10/16/2024            |                   |                   |
| 1301          | Implementar microbenchmark de accesos de memoria desordenados                                                                                                                         | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/16/2024            | 10/18/2024            |                   |                   |
| 1400          | Implementar microbenchmark de ancho de banda de copiar de buffer->textura                                                                                                             | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/21/2024            | 10/23/2024            |                   |                   |
| 1401          | Implementar microbenchmark de ancho de banda de copiar entre texturas                                                                                                                 | 5                   | 0%             | 5          |            | 0%                 |            |             |             | 10/23/2024            | 10/25/2024            |                   |                   |
| 1500          | Agregar benchmarks faltantes a herramienta CLI                                                                                                                                        | 2                   | 0%             | 2          |            | 0%                 |            |             |             | 10/28/2024            | 10/30/2024            |                   |                   |
| 1501          | Agregar benchmarks faltantes a servidor web, incluyendo alteraciones necesarias en la base de datos y cambios en la interfaz web                                                      | 10                  | 0%             | 10         |            | 0%                 |            |             |             | 10/30/2024            | 11/1/2024             |                   |                   |
| 1600          | Mejorar la documentación del repositorio para guíar desarrolladores en el uso de la herramienta CLI, ejecutar el servidor de manera local o producción y cómo agregar microbenchmarks | 4                   | 0%             | 4          |            | 0%                 |            |             |             | 11/4/2024             | 11/6/2024             |                   |                   |
| 1601          | Redactar informe final                                                                                                                                                                | 10                  | 0%             | 10         |            | 0%                 |            |             |             | 11/6/2024             | 11/8/2024             |                   |                   |

[TODO: incluir gráfica de PV, AC y EV desde el inicio del proyecto hasta la fecha reportada]

[TODO: incluir gráfica de CPI y SPI desde el inicio del proyecto hasta la fecha reportada]

\elscape

# 6. Lecciones Aprendidas

[TODO: Incluir lecciones aprendidas. Acciones, verbos, cosas que se deben hacer o no hacer a futuro.]

- **Lección 1:** Descripción de la lección aprendida.
- **Lección 2:** Descripción de la lección aprendida.
- **Lección 3:** Descripción de la lección aprendida.

