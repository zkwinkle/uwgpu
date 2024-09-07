---
toc: yes
header-includes: yes
geometry: "left=1.6cm,right=1.6cm,top=1.6cm,bottom=1.6cm"
gradient: no
underline: no
colorlinks: yes
titlepage: yes

title: Documento de Diseño
subtitle: µwgpu
author: Ignacio Vargas Campos
id: 2019053776
date: II Semestre 2024
institute: Instituto Tecnológico de Costa Rica
department: Escuela de Ingeniería en Computadores
---

# 1. Detalles del Documento

## 1.1. Fecha de la Versión y Estatus

- Fecha: 6 de septiembre de 2024
- Estatus: Versión 1.0

## 1.2. Organización

El proyecto será llevado a cabo como parte de un proyecto de investigación en el Instituto Tecnológico de Costa Rica.

## 1.3. Autor

Ignacio Vargas Campos

## 1.4. Historia de Cambios

| Versión | Fecha                  | Descripción de Cambios                 |
|---------|------------------------|----------------------------------------|
| 1.0     | 6 de septiembre de 2024| Versión inicial del documento          |


# 2. Introducción

## 2.1. Propósito

El objetivo principal de este software es proporcionar un conjunto de
microbenchmarks que permitan medir características específicas del rendimiento
de GPUs en plataformas compatibles con WebGPU y Vulkan.

Además, el software facilita a desarrolladores la creación y ejecución de sus
propios microbenchmarks personalizados a través de una biblioteca.

Esta herramienta responde a la necesidad de obtener datos de rendimiento
detallados y comparativos a través de diversas combinaciones de hardware y
plataformas, permitiendo a desarrolladores e investigadores optimizar sus
aplicaciones para diferentes escenarios y evaluar el rendimiento general de las
GPUs.

## 2.2. Alcance

El software, denominado "µwgpu," abarca el diseño, desarrollo e implementación
de microbenchmarks para evaluar las características de hardware de GPUs, así
como las interfaces necesarias para su ejecución y visualización de resultados.

El proyecto tiene dos objetivos centrales:

- Simplificar la creación de microbenchmarks para GPU de manera
multi-plataforma.
- Proveer un banco de microbenchmarks comparativos que incluyan operaciones
comunes para evaluar las capacidades del hardware en ejecución.

El software ofrecerá dos tipos de interfaces de usuario:

1. **Interfaz web:** Permite ejecutar microbenchmarks directamente desde un
navegador, recolectar datos y mostrar resultados.
2. **Interfaz de línea de comandos (CLI):** Permite ejecutar microbenchmarks de
manera nativa, compilando para Vulkan y proporcionando los resultados en
formato de texto.

## 2.3. Contexto

El software "µwgpu" se desarrollará como parte de un proyecto de investigación del Instituto Tecnológico de Costa Rica, orientado a abordar las necesidades de la comunidad de desarrollo de software gráfico, tanto en el ámbito académico como en la industria. Este proyecto surge del interés de proporcionar herramientas útiles para el análisis y optimización del rendimiento de GPUs, especialmente en entornos que utilizan tecnologías emergentes como WebGPU y Vulkan.

El producto será utilizado principalmente por desarrolladores e investigadores que buscan recolectar y analizar datos de rendimiento detallados a través de diferentes configuraciones de hardware y software. Su objetivo es facilitar el estudio de las capacidades de las GPUs, así como la creación de programas y algoritmos que puedan ejecutarse de manera eficiente en una amplia variedad de dispositivos y plataformas. Además, esta herramienta también resultará valiosa para empresas tecnológicas que buscan optimizar sus productos y para fabricantes de hardware que desean evaluar el rendimiento de sus dispositivos en diferentes escenarios.

## 2.4. Resumen

- Resumen general del contenido del documento, detallando el diseño.

## 2.5. Interesados

- **Desarrolladores e Ingenieros en GPU**: Utilizan este proyecto para
optimizar el rendimiento de aplicaciones mediante microbenchmarks específicos
de WebGPU. Buscan herramientas precisas para identificar puntos críticos de
rendimiento y mejorar la eficiencia.
- **Investigadores Académicos**: Emplean los microbenchmarks en estudios y
experimentos relacionados con el rendimiento de GPUs, buscando datos exactos
que puedan validar sus teorías y apoyar la publicación de resultados.
- **Empresas de Tecnología y Desarrollo de Software**: Implementan los
microbenchmarks para optimizar el rendimiento y la experiencia de usuario en
sus productos, identificando y solucionando problemas antes del lanzamiento.
- **Proveedores de Hardware**: Evalúan el rendimiento de sus GPUs en diversos
escenarios para ajustar y mejorar sus productos, basándose en los resultados
proporcionados por los microbenchmarks.
- **Usuarios Finales (Desarrolladores y Usuarios de Aplicaciones)**: Se
benefician indirectamente de las mejoras en el rendimiento de las aplicaciones,
lo cual afecta positivamente la calidad de su experiencia.

# 3. Referencias

- [Proyecto wgpu](https://github.com/gfx-rs/wgpu)
- [Estándar de WebGPU](https://gpuweb.github.io/gpuweb/)
- [Proyecto µVkCompute](https://github.com/google/uVkCompute)
- [Dissecting GPU Memory Hierarchy through
Microbenchmarking](https://arxiv.org/abs/1509.02308)
- [Demystifying GPU Microarchitecture through
Microbenchmarking](https://courses.cs.washington.edu/courses/cse470/24sp/readings/Demystifying_GPU_microarchitecture_through_microbenchmarking.pdf)
- [Nvidia CUDA Programming
Guide](https://developer.download.nvidia.com/compute/cuda/1.0/NVIDIA_CUDA_Programming_Guide_1.0.pdf)
- [GPU Atomic Performance Modeling with
Microbenchmarks](https://vulkan.org/user/pages/09.events/vulkanised-2024/vulkanised-2024-devon-mckee.pdf)

# 4. Glosario

- **Microbenchmark**: Prueba destinada a medir el rendimiento de una
característica específica de bajo nivel. A diferencia de un "benchmark"
general, un microbenchmark se centra en pruebas específicas, generalmente a
nivel de componentes.
- **GPU Compute**: Uso de la GPU para realizar cálculos computacionales de
propósito general.
- **GPU**: Unidad de Procesamiento Gráfico. **CLI**: Interfaz de Línea de
Comandos.

# 5. Perspectivas de diseño

## 5.1. Contexto

- Design concers de la perspectiva

- Design elements, defined by that viewpoint, specifically the types of design
entities, attributes, relationships, and constraints introduced by that
viewpoint or used by that viewpoint (which may have been defined elsewhere).
These elements may be realized by one or more design languages; Formal or
informal consistency and completeness tests to be applied to the view;

- Evaluation or analysis techniques to be applied to a view; and

- Heuristics, patterns, or other guidelines to assist in construction or
synthesis of a view.

- An SDD shall include a rationale for the selection of each selected
viewpoint.

- Each design entity shall have a name, a type, and purpose.

# 6. Apéndice - Alternativas de diseño

## 6.1. Creación de bindings/buffers

- Usar naga para inspeccionar shader y crear automáticamente

- Permitir múltiples bind groups
