#!/usr/bin/execlineb -P
background {
  s6-sleep 5 /postinit.sh
}

s6-echo -- "Postinit Script delayed by 5 seconds"
