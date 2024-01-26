let ConfigString = fun label value =>
  if std.is_string value then
    if std.string.length value > 0 then
      value
    else
      std.contract.blame_with_message "Field cannot be an empty string" label
  else
    std.contract.blame_with_message "Expected value to be a string" label
in

let SSHTarget = {
    target
      | ConfigString
      | doc "IP Address and host of the host. e.g root@localhost:22",
    id
      | ConfigString
      | doc "Identifier for the remote connection. e.g lilit",
  atc-file-path
      | String
      | optional
      | doc "Osquery ATC absolute path"
  } in

let Authentication = {
  username | ConfigString,
  password | ConfigString,
} in

let Supplier =
  {
    type
      | std.enum.TagOrString
      | [| 'osquery, |]
      | doc "Enum values of supplier name",
    mode
      | std.enum.TagOrString
      | [| 'local, 'remote |]
      | doc "Start the supplier on local or remote machines",
    ssh-targets
      | Array SSHTarget
      | optional,
    auth
      | Array Authentication
      | doc "Authentication for supplier",
    atc-file-path
      | String
      | optional
      | doc "Osquery ATC absolute path"
  } in

let ConfigSchema =
  {
    addr
      | String
      | optional
      | default
      | doc "Address to start pgp on"
      = "127.0.0.1:5432",
    metrics
      | Number
      | optional,
    health
      | Number
      | optional,
    suppliers | { _: Supplier },
  } in

{
  config = {
    port = 5556,
    metrics = 3144,
    health = 3155,
    suppliers = {
      local-supplier = {
        type = "osquery",
        mode = "local",
        auth = [
          {
            username = "baasit",
            password = "supplier",
          },
        ],
      },
      hetzner-atc = {
        type = "osquery",
        mode = "local",
        atc-file-path = "./hetzner-atc.json",
        auth = [
          {
            username = "baasit",
            password = "supplier",
          },
        ],
      },
      supplier1 = {
        type = "osquery",
        mode = "remote",
        ssh-targets = [
          {
            id = "one",
            target = "oshuporu@157.245.40.97:22"
          },
          {
            id = "me",
            target = "lilit@localhost:22"
          },
          {
            id = "do_test",
            target = "root@128.199.1.17:22"
          }
        ],
        auth = [
          {
            username = "baasit",
            password = "supplier",
          },
        ],
      }
    }
  } | ConfigSchema
}
