## CLEO and jq

jq is a command line tool to pipe outputs from json into pretty json or extract values.
That is how jq can automate cli-applications.

#### Basic jq 
- jq -r removes " from strings.
- [] constructs an array
- {} constructs an object

e.g. `jq  '[ { "name:" .[].name, "id:" .[].id } ]'` or: `jq '[ .[] | { title, name } ]'`

**input**
```shell
opendut-cleo list --output=pretty-json peers
```

**output**

This output will be exemplary for the following jq commands.
```json
[
  {
    "name": "HelloPeer",
    "id": "90dfc639-4b4a-4bbb-bad3-6f037fcde013",
    "status": "Disconnected"
  },
  {
    "name": "Edgar",
    "id": "defe10bb-a12a-4ad9-b18e-8149099dd044",
    "status": "Connected"
  },
  {
    "name": "SecondPeer",
    "id": "c3333d4e-9b1a-4db5-9bfa-7a0a40680f1a",
    "status": "Disconnected"
  }
]
```
**input**
```shell
opendut-cleo list --output=json peers | jq '[.[].name]'
```


**output**    
jq extracts the names of every json element in the list of peers.
```json
[
  "HelloPeer",
  "Edgar", 
  "SecondPeer" 
]
```
which can also be put into an array with `cleo list --output=json peers | jq '[.[].name']`

**input**
```shell
opendut-cleo list --output=json peers | jq '[.[] | select(.status=="Disconnected")]'
```

**output**    

```json
[
    {
      "name": "HelloPeer",
      "id": "90dfc639-4b4a-4bbb-bad3-6f037fcde013",
      "status": "Disconnected"
    },
    {
      "name": "SecondPeer",
      "id": "c3333d4e-9b1a-4db5-9bfa-7a0a40680f1a",
      "status": "Disconnected"
    }
]
```

**input**
```shell
opendut-cleo list --output=json peers | jq '.[] | select(.status=="Connected") | .id' | xargs -I{} cleo describe peer -i {}
```
**output**
```
Peer: Edgar
  Id: defe10bb-a12a-4ad9-b18e-8149099dd044
  Devices: [device-1, The Device, Another Device, Fubar Device, Lost Device]
```

**Get the number of the peers**
```shell
opendut-cleo list --output=json peers | jq 'length'
```

**Sort peers by name**
```shell
opendut-cleo list --output=json peers | jq 'sort_by(.name)'
```
