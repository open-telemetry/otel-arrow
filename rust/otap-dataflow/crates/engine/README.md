# Dataflow Engine

Status: **WIP**

## Nodes

```mermaid
graph LR
    RecvControlIn[Control Channel] --> Receiver
    Receiver((Receiver))
    Receiver --> RecvControlOut[Control Channel]
    Receiver --> RecvDataOut[Data Channel]
    class Receiver node
    class RecvControlIn,RecvControlOut controlQueue
    class RecvDataOut dataQueue
    
    ProcControlIn[Control Channel] --> Processor
    ProcDataIn[Data Channel] --> Processor
    Processor((Processor))
    Processor --> ProcControlOut[Control Channel]
    Processor --> ProcDataOut[Data Channel]
    class Processor node
    class ProcControlIn,ProcControlOut controlQueue
    class ProcDataIn,ProcDataOut dataQueue

    ExControlIn[Control Channel] --> Exporter
    ExDataIn[Data Channel] --> Exporter
    Exporter((Exporter))
    Exporter --> ExControlOut[Control Channel]
    class Exporter node
    class ExControlIn,ExControlOut controlQueue
    class ExDataIn,ExDataOut dataQueue

    classDef controlQueue fill:#ffdddd,stroke:#cc0000,color:#000,stroke-width:0px,font-size:10px,padding:0px
    classDef dataQueue fill:#ddffdd,stroke:#009900,color:#000,stroke-width:0px,font-size:10px,padding:0px
    classDef node fill:#4a90e2,color:#ffffff,stroke-width:0px,font-size:12px
```