# Overview

```bob

       ,-.              +-------+         
       `-'              |       |          
       /|\   -----+---->|  LEA  +---+     
        |         |     |       |   |      
       / \        |     +-------+   |     +-------+ 
                  |                 |     |       |
      User        |                 +---->| CARL  |
                  |                 |     |       |  
                  |     +-------+   |     +-------+
       ,-.        +---->|       |   |       ^   ^
       `-'              | CLEO  +---+       |   |
       /|\    --------->|       |           |   |
        |               +-------+           |   |
       / \                                  |   |
                                            |   |
      "CI/CD"                               |   |
                              +-------------+   +-------------+
                              |                               |
                              |                               |
                              |                               |                             
                              v                               v      
                         +----+----+                     +----+----+ 
                         |         |                     |         | 
                         |  EDGAR  |                     |  EDGAR  | 
                         |         |                     |         | 
                         +----+----+                     +----+----+ 
                              |                               |
                       +------+------+                        |
                       |             |                        |
                   +---+---+     +---+---+                +---+---+
                   |       |     |       |                |       |
                   |  DUT  |     |  DUT  |                |  DUT  |
                   |       |     |       |                |       |
                   +-------+     +-------+                +-------+
```

### Components
- **CARL** (Control And Registration Logic)
- **EDGAR** (Edge Device Global Access Router)
- **LEA** (Leasing ECU Access)
- **CLEO** (Command-Line ECU Orchestrator)
- **DUT** (Device under test)


