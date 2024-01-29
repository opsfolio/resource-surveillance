# Universal Data Infrastructure PostgreSQL Wire Proxy (UDI-PGP)

UDI PostgreSQL Proxy, designed for remote SQL management, is a command-line tool that sets up a server acting as a PostgreSQL interface. It channels SQL queries to other CLI commands, known as _SQL Suppliers_.

## Usage Guide

The UDI PostgreSQL Proxy enables the execution of SQL commands through various modes of operation, including local and remote. Below are detailed instructions for each mode.

### Osquery Usage

#### Local Mode

In local mode, the `surveilr udi pgp` command initiates a PostgreSQL proxy server on your specified port. This allows you to execute SQL queries using PostgreSQL clients like `psql`.

**Example Command:**
```bash
surveilr udi pgp -a 127.0.0.1:5555 -u john -p doe -i supplier-one osquery local
```

- `-a 127.0.0.1:5555`: Specifies the address and port for the proxy server.
- `-u john -p doe`: Sets the username and password.
-  `-i supplier-one`: Sets the supplier identifier or name.

To query the server, use `psql` as follows:
```bash
psql -h 127.0.0.1 -p 5555 -U john -d "supplier-one" -c "SELECT cpu_type, cpu_brand, hardware_vendor, hardware_model FROM system_info"
```

#### Remote Mode

To utilize the remote mode, you must first ensure that SSH Authentication is set up correctly, as `surveilr` currently does not support direct SSH key passing.

**SSH Setup for Bash:**
- For users with a Bash terminal, append the following script to the `.bashrc` file. This script checks for the existence of `~/.ssh/id_rsa` and, if found, adds it to the SSH Authentication agent.

```bash
# Start SSH agent and add keys if ~/.ssh/id_rsa exists
if test -f ~/.ssh/id_rsa; then
    eval "$(ssh-agent)" > /dev/null
    if ! ssh-add -l > /dev/null; then
        ssh-add ~/.ssh/id_rsa > /dev/null 2>&1
    fi
fi
```
**Example Remote Mode Command:**
```bash
surveilr udi pgp -a 127.0.0.1:5555 -u john -p doe -i second-supp osquery remote -s "user@127.0.0.1:22,john" -s "lilit@website.com:22,doe"
```
- The `-i second-supp` specifies the supplier identifier or na,e.
- The `-s` flag specifies remote hosts and credentials.

To execute a query remotely:
```bash
psql -h 127.0.0.1 -p 5555 -U john -d "second-supp" -c "SELECT cpu_type, cpu_brand, hardware_vendor, hardware_model FROM system_info"
```

#### Using ATCs (Auto Table Construction)

The ATC mode allows the execution of predefined queries stored in JSON format.

**Example ATC Mode Command:**
```bash
surveilr udi pgp -u john -p doe osquery local -a ./hetzner/hetzner.omc.sqla.osquery-atc.json
```
- `-a ./hetzner/hetzner.omc.sqla.osquery-atc.json`: Specifies the path to the ATC file.

To run a query using ATC:
```bash
psql -h 127.0.0.1 -p 5432 -U john -c "SELECT * FROM person"
```

## Configuration File Usage
UDI-PGP has been enhanced to support the use of configuration files, offering an alternative to passing arguments and parameters directly. This feature is particularly beneficial when working with multiple suppliers. When a configuration file is provided as an optional parameter, UDI-PGP prioritizes the settings within this file, disregarding any other command-line arguments. The configuration files can be in either Nickel or JSON format. This approach includes automatic schema checking, along with error detection and remediation processes.

For an illustrative example of a Nickel configuration file, please refer to the following [link](https://nickel-lang.org/getting-started/).

**Example Configuration File Command:**
```bash
surveilr udi pgp -c ./support/config-full.ncl
```

### Updating Configuration

UDI-PGP allows for dynamic configuration updates even while the proxy server is operational. This is achieved through the use of SET statements targeting specific keys, and is currently supported in NCL format. For instance, the command SET udi_pgp_serve_ncl_supplier = {...} can be used to introduce a new supplier. If a supplier configuration is updated, UDI-PGP automatically recognizes these changes, adjusting the parameters for that specific supplier and acknowledging the addition of new suppliers.
To modify operational aspects such as health and port addresses, the udi_pgp_serve_ncl_core key is utilized.

For comprehensive examples demonstrating these update processes, please refer to the following [resource](../../support/test-e2e.sql). 