use heck::*;
use tera::{Kwargs, State, Tera};
use tera_contrib::*;

pub fn register_all(tera: &mut Tera) {
    tera.register_filter("upper_camel_case", upper_camel_case);
    tera.register_filter("camel_case", camel_case);
    tera.register_filter("snake_case", snake_case);
    tera.register_filter("kebab_case", kebab_case);
    tera.register_filter("shouty_snake_case", shouty_snake_case);
    tera.register_filter("title_case", title_case);
    tera.register_filter("shouty_kebab_case", shouty_kebab_case);
    tera.register_filter("b64_encode", base64::b64_encode);
    tera.register_filter("b64_decode", base64::b64_decode);
    tera.register_filter("date", dates::date);
    tera.register_function("now", dates::now);
    tera.register_test("before", dates::is_before);
    tera.register_test("after", dates::is_after);
    tera.register_filter("filesize_format", filesize_format::filesize_format);
    tera.register_filter("format", format::format);
    tera.register_filter("json_encode", json::json_encode);
    tera.register_function("get_random", rand::get_random);
    tera.register_filter("shuffle", rand::shuffle);
    tera.register_filter("striptags", regex::striptags);
    tera.register_filter("spaceless", regex::spaceless);
    tera.register_filter("regex_replace", regex::RegexReplace::default());
    tera.register_test("matching", regex::Matching::default());
    tera.register_filter("slug", slug::slug);
    tera.register_filter("urlencode", urlencode::urlencode);
    tera.register_filter("urlencode_strict", urlencode::urlencode_strict);
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

        // filters
        for (tpl, expected) in [
            ("{{ 'foo bar' | upper_camel_case }}", "FooBar"),
            ("{{ 'foo bar' | camel_case }}", "fooBar"),
            ("{{ 'foo bar' | snake_case }}", "foo_bar"),
            ("{{ 'foo bar' | kebab_case }}", "foo-bar"),
            ("{{ 'foo bar' | shouty_snake_case }}", "FOO_BAR"),
            ("{{ 'foo bar' | title_case }}", "Foo Bar"),
            ("{{ 'foo bar' | shouty_kebab_case }}", "FOO-BAR"),
            ("{{ 'hello world' | b64_encode }}", "aGVsbG8gd29ybGQ="),
            ("{{ 'aGVsbG8gd29ybGQ=' | b64_decode }}", "hello world"),
            ("{{ '2024-01-15' | date(format='%Y') }}", "2024"),
            ("{{ 1024 | filesize_format }}", "1 KiB"),
            ("{{ 3.14159 | format(spec='.2') }}", "3.14"),
            ("{{ '<b>Joel</b>' | striptags }}", "Joel"),
            (r"{{ '<p>\n<a> </a>\r\n </p>' | spaceless }}", "<p><a></a></p>"),
            (r"{{ 'abc123def456' | regex_replace(pattern='\\d+', rep='') }}", "abcdef"),
            ("{{ 'Foo & Bar!' | slug }}", "foo-bar"),
            ("{{ 'foo/bar?baz=1' | urlencode }}", "foo/bar%3Fbaz%3D1"),
            ("{{ 'a/b' | urlencode_strict }}", "a%2Fb"),
        ] {
            assert_eq!(tera.render_str(tpl, &ctx, false).unwrap(), expected, "{tpl}");
        }

        // functions
        tera.render_str("{{ now() }}", &ctx, false).unwrap();
        tera.render_str("{{ get_random(start=1, end=100) }}", &ctx, false).unwrap();

        // tests
        for (tpl, expected) in [
            ("{{ 'abc' is matching(pat='^a') }}", "true"),
            ("{{ 'abc' is matching(pat='^b$') }}", "false"),
            ("{{ '2024-01-01' is before(other='2024-06-01') }}", "true"),
            ("{{ '2024-06-01' is before(other='2024-01-01') }}", "false"),
            ("{{ '2024-01-01' is before(other='2024-01-01', inclusive=true) }}", "true"),
            ("{{ '2024-06-01' is after(other='2024-01-01') }}", "true"),
            ("{{ '2024-01-01' is after(other='2024-06-01') }}", "false"),
            ("{{ '2024-01-01' is after(other='2024-01-01', inclusive=true) }}", "true"),
        ] {
            assert_eq!(tera.render_str(tpl, &ctx, false).unwrap(), expected, "{tpl}");
        }
    }
}
