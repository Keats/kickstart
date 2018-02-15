{% if use_clap %}
#[macro_use]
extern crate clap;
{% endif %}

fn main() {
    {% if use_clap -%}
    let app = App::new("{{bin_name}}")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!());

    let matches = app.get_matches();
    {%- else -%}
    println!("Hello, world!");
    {%- endif %}
}
