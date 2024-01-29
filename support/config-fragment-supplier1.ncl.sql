SET udi_pgp_serve_ncl_supplier = '
  let new-supplier = { 
    type = "osquery",
    mode = "remote",
    ssh-targets = [
      {
        id = "one",
        host = "157.245.40.97",
        port = 22,
        user = "oshuporu"
      },
      {
        id = "me",
        host = "localhost",
        port = 222,
        user = "lilit"
      },
      {
        id = "do_test",
        host = "128.199.1.17",
        port = 22,
        user = "root"
      },
    ],
    auth = [
      {
        username = "baasit",
        password = "supplier",
      },
    ],
  } in new-supplier';