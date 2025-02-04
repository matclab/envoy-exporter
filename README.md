# envoy-exporter
[Prometheus](https://prometheus.io) exporter for [Enphase Envoy Gateway](https://enphase.com/en-us/products-and-services/envoy).

This exporter is not endorsed by or approved by Enphase.

Features:

* Can poll multiple Envoy gateways in a single metrics call
* Exports Envoy data such as current watts, today's watt hours and lifetime watt hours
* Exports individual inverter data such as each serial number and last-reported watts
* Export consumption data as measured by envoy

## Usage

```
envoy-exporter [config_file]
```

The format of `config_file` is shown in the `etc` directory.

### Docker 
```
docker build -t envoy-exporter .
docker run -it --rm -p 9433:9433 -v ${PWD}/etc/config.toml:/config.toml envoy-exporter
```

An ARM 64 bit docker image suitable for the RPI is available at matclab/envoy-exporter.

The password for the Envoy is typically the last six characters of the serial
number. The serial number is available from the Envoy's public web interface.

See [This web
page](https://web.archive.org/web/20220703201559/https://thecomputerperson.wordpress.com/2016/08/03/enphase-envoy-s-data-scraping/)
for more information about the API and the authentication.

Arch Linux users can install the
[envoy-exporter-git](https://aur.archlinux.org/packages/envoy-exporter-git/)
package from AUR for a security-focused systemd unit with minimal permissions.
