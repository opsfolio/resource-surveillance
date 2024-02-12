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
    host
      | ConfigString
      | doc "e.g. localhost",
    port
      | Number
      | doc "Port to bind SSH connection to"
      | optional
      | default
      = 22,
    user
      | String
      | doc "Username for the ssh connection",
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
      | String
      | optional
      | default
      = "127.0.0.1:3144",
    health
      | String
      | optional
      | default
      = "127.0.0.1:3155",
    verbose
      | Bool
      | default
      = false,
    admin-state-fs-path
      | String
      | optional
      | default
      = "resource-surveillance-admin.sqlite.db",
    suppliers | { _: Supplier },
  } in

{
  config = {
    addr = "127.0.0.1:7777",
    metrics = "127.0.0.1:6666",
    health = "127.0.0.1:5566",
    verbose = false,
    suppliers = {}
  } | ConfigSchema
}
