#!/usr/bin/env fish
# Note: Run this script with the following command: 
# 'source init-env.fish' in your fish shell.
set -x ZT_PRIVATE_KEY (cat zitadel-newline-edited-private-key.pem)
set -x ZT_PUBLIC_KEY (cat zitadel-openssl-generated-public-key.pem)
