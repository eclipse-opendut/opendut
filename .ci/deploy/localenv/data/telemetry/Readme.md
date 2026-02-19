# Telemetry

## Checks

The following checks should be performed when changing the telemetry stack:

* All kinds of telemetry data is visible in Grafana: 
  * Logs
  * Metrics
  * Traces
* All telemetry data is tagged with the correct labels:
  * OpenTelemetry `service.name` → service_name in Grafana (indexed label)
  * `service_instance_id` (indexed label)
  * `severity_text` (structured metadata)
* Filtering by service name works as expected
* Custom log filters work: 
  * `severity_text=TRACE`
  * `service_instance_id=edgar-525b369f-8abb-4b49-8046-25948936ad6c`
  * `service_name=opendut-carl`
  * `scope_name=opendut_carl::manager::peer_messaging_broker`


## Grafana configuration

* Read label [best practices](https://grafana.com/docs/loki/latest/get-started/labels/bp-labels/)


### OpenDuT logs dashboard

* Standard log search:
    ```
    {service_name=~"$service_name"} | logfmt
    ```
* Filter by error level:
    ```
    {service_name=~"$service_name"} | logfmt | severity_text = `ERROR`
    ```

### Table of log levels

Sum all logs while preserving label severity_text:
```
sum by(severity_text) (count_over_time({service_name=~"$service_name"} | logfmt [$__auto]))
sum by(severity_text) (count_over_time({service_name=~"$service_name"} |= `$query_filter` | logfmt [$__auto]))
```
Transformations:
* 1 - Reduce: 
  * Mode: `Series to rows`, Calculations: Total
* 2 - Extract fields: Source `Field`
* 3 - Organize fields by name: Remove JSON Field, rename `severity_text` → `Level`

### Table of log lines

Table of the `Number of lines per service`.

* It uses the following query:
    ```
    sum by(service_name) (count_over_time({service_name=~"$service_name"} |= `$query_filter` | logfmt [$__auto]))
    ```
* Alternative queries:
    ```
    sum by(service_name) (count_over_time({service_name=~".+"} | logfmt [$__auto]))
    sum by(service_name) (count_over_time({service_name=~"$service_name"} | logfmt [$__auto]))
    ```

Transformations:
* 1 - Reduce:
    * Mode: `Series to rows`, Calculations: Total
* 2 - Extract fields: Source `Field`
* 3 - Organize fields by name: Remove JSON Field, rename `severity_text` → `Level`
* 4 - Sort by: Field `Total`, reversed


## Alloy

* [Forward logs to loki](https://grafana.com/docs/alloy/latest/reference/components/loki/loki.process/)

## Labels

https://grafana.com/docs/loki/latest/get-started/labels/#default-labels-for-all-users
