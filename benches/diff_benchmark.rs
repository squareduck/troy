#[macro_use]
extern crate criterion;
extern crate troy;

use troy::diff::diff;
use troy::tags::*;

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let old = div().class("app")
        .child(div().class("header")
            .child(p().text("Todo List"))
            .child(div().class("user")
                .child(p().class("name").text("John Doe"))
                .child(p().class("total tasks").text("12"))
                .child(p().class("done tasks").text("6"))
            ),
        )
        .child(div().class("tasks")
            .child(p().key("1").child(p().class("status").text("done")).text("Task one"))
            .child(p().key("2").child(p().class("status").text("done")).text("Task two"))
            .child(p().key("3").child(p().class("status").text("done")).text("Task three"))
            .child(p().key("4").child(p().class("status").text("done")).text("Task four"))
            .child(p().key("5").child(p().class("status").text("done")).text("Task five"))
            .child(p().key("6").child(p().class("status").text("done")).text("Task six"))
            .child(p().key("7").child(p().class("status").text("undone")).text("Task seven"))
            .child(p().key("8").child(p().class("status").text("undone")).text("Task eight"))
            .child(p().key("9").child(p().class("status").text("undone")).text("Task nine"))
            .child(p().key("10").child(p().class("status").text("undone")).text("Task ten"))
            .child(p().key("11").child(p().class("status").text("undone")).text("Task eleven"))
            .child(p().key("12").child(p().class("status").text("undone")).text("Task twelve"))
        )
        .done();

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let new = div().class("app")
        .child(div().class("header")
            .child(p().text("Todo List"))
            .child(div().class("user")
                .child(p().class("name").text("John Doe"))
                .child(p().class("total tasks").text("8"))
                .child(p().class("done tasks").text("3"))
            ),
        )
        .child(div().class("tasks")
            .child(p().key("5").child(p().class("status").text("done")).text("Task five"))
            .child(p().key("13").child(p().class("status").text("done")).text("Task thirteen"))
            .child(p().key("6").child(p().class("status").text("done")).text("Task six"))
            .child(p().key("2").child(p().class("status").text("undone")).text("Task two"))
            .child(p().key("1").child(p().class("status").text("undone")).text("Task one"))
            .child(p().key("14").child(p().class("status").text("undone")).text("Task fourteen"))
            .child(p().key("9").child(p().class("status").text("undone")).text("Task nine"))
            .child(p().key("11").child(p().class("status").text("undone")).text("Task eleven"))
        )
        .done();

    c.bench_function("diff", move |b| b.iter(|| diff(&old, &new)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
