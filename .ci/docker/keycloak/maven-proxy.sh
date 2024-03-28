#!/usr/bin/env bash

# see https://maven.apache.org/settings.html

# global maven settings
# MAVEN_PROXY_CONFIG="/usr/share/maven/conf/settings.xml"
# user maven settings
MAVEN_PROXY_CONFIG="$HOME/.m2/settings.xml"

if [ -z "$HTTP_PROXY" ]; then
  if [ -e "$MAVEN_PROXY_CONFIG" ]; then
    echo "Removing proxy configuration for maven at $MAVEN_PROXY_CONFIG"
    rm "$MAVEN_PROXY_CONFIG"
  fi
else
  echo "Creating proxy configuration for maven at $MAVEN_PROXY_CONFIG"

  MAVEN_PROXY_HOST=$(echo "$HTTP_PROXY" | sed -e 's#http://##' | awk -F':' '{print $1}')
  MAVEN_PROXY_PORT=$(echo "$HTTP_PROXY" | sed -e 's#http://##' | awk -F':' '{print $2}')

  mkdir -p "$HOME"/.m2
  cat > "$MAVEN_PROXY_CONFIG" << EOF
<settings>
  <proxies>
    <proxy>
        <id>myhttpproxy</id>
        <active>true</active>
        <protocol>http</protocol>
        <host>$MAVEN_PROXY_HOST</host>
        <port>$MAVEN_PROXY_PORT</port>
        <nonProxyHosts>localhost</nonProxyHosts>
    </proxy>
  </proxies>
</settings>
EOF

fi
