# {{ project_name | capitalize }}

## Database

The project uses {{database}} {% if pg_version %}{{pg_version}}{% endif %}.
{% if spa -%}
{%- if js_framework == "React" -%}
The frontend is a SPA built in React.
{%- elif js_framework == "Angular" -%}
The frontend is a SPA built in Angular.
{%- elif js_framework == "Vue" -%}
The frontend is a SPA built in Vue.
{%- else -%}
The frontend is a SPA built in vanilla JavaScript.
{% endif %}
{% if typescript -%}
It is written in TypeScript.
{%- endif -%}

{% endif %}

## Authentication

{% if auth_method == "jwt" -%}
A JWT will be sent in every requests' Authorization header.
{%- elif auth_method == "sessions" -%}
A cookie will be set upon logging.
{%- else -%}
No authentication is needed.
{%- endif %}.

{% if sentry %}
## Error reporting
Errors will be reported to Sentry once you create a project and add the DSN in the config.
{% endif %}
