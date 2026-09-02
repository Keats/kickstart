use heck::*;
use tera::{Kwargs, State, Tera};

pub fn register_all(tera: &mut Tera) {
    tera.register_filter("upper_camel_case", upper_camel_case);
    tera.register_filter("camel_case", camel_case);
    tera.register_filter("snake_case", snake_case);
    tera.register_filter("kebab_case", kebab_case);
    tera.register_filter("shouty_snake_case", shouty_snake_case);
    tera.register_filter("title_case", title_case);
    tera.register_filter("shouty_kebab_case", shouty_kebab_case);
    tera.register_filter("slug", tera_contrib::slug::slug);
    tera.register_filter("date", tera_contrib::dates::date);
    tera.register_function("now", tera_contrib::dates::now);
}

pub fn upper_camel_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_upper_camel_case()
}

pub fn camel_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_lower_camel_case()
}

pub fn snake_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_snake_case()
}

pub fn kebab_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_kebab_case()
}

pub fn shouty_snake_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_shouty_snake_case()
}

pub fn title_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_title_case()
}

pub fn shouty_kebab_case(value: &str, _: Kwargs, _: &State) -> String {
    value.to_shouty_kebab_case()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_succeeded() {
        let mut tera = Tera::default();
        register_all(&mut tera);
        let ctx = tera::Context::new();
        for (tpl, expected) in [
            ("{{ 'foo bar' | upper_camel_case }}", "FooBar"),
            ("{{ 'foo bar' | camel_case }}", "fooBar"),
            ("{{ 'foo bar' | snake_case }}", "foo_bar"),
            ("{{ 'foo bar' | kebab_case }}", "foo-bar"),
            ("{{ 'foo bar' | shouty_snake_case }}", "FOO_BAR"),
            ("{{ 'foo bar' | title_case }}", "Foo Bar"),
            ("{{ 'foo bar' | shouty_kebab_case }}", "FOO-BAR"),
            ("{{ 'Foo & Bar!' | slug }}", "foo-bar"),
            ("{{ '2024-01-15' | date(format='%Y') }}", "2024"),
        ] {
            assert_eq!(tera.render_str(tpl, &ctx, false).unwrap(), expected, "{tpl}");
        }
        tera.render_str("{{ now() }}", &ctx, false).unwrap();
    }
}
