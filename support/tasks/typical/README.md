## Surveilr Ingest Tasks for Device Evidence Collection

Prepare the device-* JSONL files here ,  pipe the respective queries to deno shell task via `surveilr ingest tasks`   to produce independent RSSD SQLite DB. 

#### Prerequisites
##### 1. Steampipe

Steampipe installation steps:

We need sudo user permission for installing Steampipe

```
$ sudo /bin/sh -c "$(curl -fsSL https://raw.githubusercontent.com/turbot/steampipe/main/install.sh)"
```

Plugin installation samples:-

```
$ steampipe plugin install digitalocean
$ steampipe plugin install aws
$ steampipe plugin install theapsgroup/keycloak
$ steampipe plugin install theapsgroup/gitlab
```

Steampipe plugin details are stored in this path

```
$ ~/.steampipe/config/

aws.spc
default.spc
digitalocean.spc
gitlab.spc
keycloak.spc
```

Sample format of the config files are as follows:

---------------------------------------------
***aws.spc***
```
connection "aws" {
  plugin = "aws"
  regions = ["us-east-1"]
  access_key = "AKxxxxxxxxxxxxxxxY5H"
  secret_key = "fS/07WxxxxxxxxxxmvalMW7t"
}
```

---------------------------------------------
***digitalocean.spc***
```
connection "digitalocean" {
  plugin  = "digitalocean"
  token = "dop_v1_8xxxxxxxxxxxxxxxxxxxxxxxxxxx0e56a"
}
```
---------------------------------------------

##### 2. Cnquery

Cnquery installation steps:

Install cnquery with the installation script:

Linux and macOS
```
bash -c "$(curl -sSL https://install.mondoo.com/sh)"
```

To run standalone queries in your shell, use the cnquery run command:
```
$ cnquery run TARGET -c "QUERY"
```

For example, this command runs a query against your local system. It lists the services installed and whether each service is running:
```
$ cnquery run local -c "services.list { name running }"
```

For AWS access need to have authenticated aws-cli configured.

##### 3. Osquery

```bash
OSQ_VERSION=`curl -fsSL https://api.github.com/repos/osquery/osquery/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")'`
OSQ_APT_CACHE=/var/cache/apt/archives
OSQ_DEB_FILE=osquery_${OSQ_VERSION}-1.linux_amd64.deb
sudo curl -fsSL -o $OSQ_APT_CACHE/$OSQ_DEB_FILE https://pkg.osquery.io/deb/$OSQ_DEB_FILE
sudo dpkg -i $OSQ_APT_CACHE/$OSQ_DEB_FILE
```

###  Independent RSSD DB generation from inside each device terminal

```bash
curl -sL https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-evidence-collector.sh
```