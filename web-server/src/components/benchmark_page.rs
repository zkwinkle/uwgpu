use maud::{html, Markup, Render};

/// A page for a specific benchmark
#[derive(Clone)]
pub enum MicrobenchmarkPage {
    Matmul,
    Reduction,
    Convolution,
    Scan,
    BufferSequential,
    BufferShuffled,
    BufferToTexture,
    TextureToTexture,
}

impl Render for MicrobenchmarkPage {
    fn render(&self) -> Markup {
        let title = self.title();

        html! {
            header { h1 { (title) } }
            p { (self.description()) }
            p { "Click the \"Start\" button to execute the microbenchmark suite. For more accurate results please close all other applications." }
            button id=(format!("run_{}_microbenchmark", title)){ "Start" }

            div class="disable-checkbox" {
                input type="checkbox" id=(format!("disable_{}_data_collection", title));
                label for=(format!("disable_{}_data_collection", title)) {
                "Select this checkbox to opt out of data collection. Benchmark results contribute to a growing database of performance data. Please consider submitting your data to support this project."
            }
            }
            div class="execution-results" {
                header { h3 {"Execution Results"} }
            // Example, will get added programatically when benchmarks are actually being ran
                h4 {"Workgroup size: 1x256"}
                p {"Total time spent: 0.286s"}
                p {"Time per iteration: 286.137ms"}
                p {"FLOPS: 7.501e9"}
                h4 {"..."}
            }
        // TODO: Historical data component
        }
    }
}

impl MicrobenchmarkPage {
    fn title(&self) -> &'static str {
        match self {
            MicrobenchmarkPage::Matmul => "Matrix Multiplication",
            MicrobenchmarkPage::Reduction => "Reduction",
            MicrobenchmarkPage::Convolution => "Convolution",
            MicrobenchmarkPage::Scan => "Scan",
            MicrobenchmarkPage::BufferSequential => {
                "Sequential Buffer Memory Access"
            }
            MicrobenchmarkPage::BufferShuffled => {
                "Shuffled Buffer Memory Accesses"
            }
            MicrobenchmarkPage::BufferToTexture => {
                "Memory Copy From Buffer To Texture"
            }
            MicrobenchmarkPage::TextureToTexture => {
                "Memory Copy From Texture To Texture"
            }
        }
    }

    fn description(&self) -> &'static str {
        match self {
            MicrobenchmarkPage::Matmul => "This microbenchmark tests the performance of multiplying two 1024x1024 matrices of 32bit floats together.",
            MicrobenchmarkPage::Reduction => todo!(),
            MicrobenchmarkPage::Convolution => todo!(),
            MicrobenchmarkPage::Scan => todo!(),
            MicrobenchmarkPage::BufferSequential => {
                "This microbenchmark tests the performance of accessing buffer elements in a sequential manner."
            }
            MicrobenchmarkPage::BufferShuffled => {
                todo!()
            }
            MicrobenchmarkPage::BufferToTexture => {
                todo!()
            }
            MicrobenchmarkPage::TextureToTexture => {
                todo!()
            }
        }
    }

    pub const fn path(&self) -> &'static str {
        match self {
            MicrobenchmarkPage::Matmul => "/matmul",
            MicrobenchmarkPage::Reduction => "/reduction",
            MicrobenchmarkPage::Convolution => "/convolution",
            MicrobenchmarkPage::Scan => "/scan",
            MicrobenchmarkPage::BufferSequential => "/buffer_sequential",
            MicrobenchmarkPage::BufferShuffled => "/buffer_shuffled",
            MicrobenchmarkPage::BufferToTexture => "buffer_to_texture",
            MicrobenchmarkPage::TextureToTexture => "texture_to_texture",
        }
    }
}
