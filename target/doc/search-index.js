var searchIndex = {};
searchIndex['telemetry'] = {"items":[[0,"","telemetry","Telemetry is a mechanism used to capture metrics in an application,\nto later store the data locally or upload it to a server for\nstatistical analysis.",null,null],[3,"Service","","",null,null],[4,"SerializationFormat","","",null,null],[13,"SimpleJson","","",0,null],[0,"plain","","",null,null],[3,"Ignoring","telemetry::plain","",null,null],[3,"Flag","","",null,null],[3,"Linear","","",null,null],[3,"Count","","",null,null],[3,"Enum","","",null,null],[8,"Histogram","","",null,null],[11,"record","","",1,{"inputs":[{"name":"histogram"},{"name":"t"}],"output":null}],[10,"record_cb","","",1,{"inputs":[{"name":"histogram"},{"name":"f"}],"output":null}],[11,"new","","",2,{"inputs":[{"name":"ignoring"}],"output":{"name":"ignoring"}}],[11,"record_cb","","",2,{"inputs":[{"name":"ignoring"},{"name":"f"}],"output":null}],[11,"record_cb","","",3,{"inputs":[{"name":"flag"},{"name":"f"}],"output":null}],[11,"new","","",3,{"inputs":[{"name":"flag"},{"name":"service"},{"name":"string"}],"output":{"name":"flag"}}],[11,"record_cb","","",4,{"inputs":[{"name":"linear"},{"name":"f"}],"output":null}],[11,"new","","",4,{"inputs":[{"name":"linear"},{"name":"service"},{"name":"string"},{"name":"u32"},{"name":"u32"},{"name":"usize"}],"output":{"name":"linear"}}],[11,"record_cb","","",5,{"inputs":[{"name":"count"},{"name":"f"}],"output":null}],[11,"new","","",5,{"inputs":[{"name":"count"},{"name":"service"},{"name":"string"}],"output":{"name":"count"}}],[11,"record_cb","","",6,{"inputs":[{"name":"enum"},{"name":"f"}],"output":null}],[11,"new","","",6,{"inputs":[{"name":"enum"},{"name":"service"},{"name":"string"},{"name":"usize"}],"output":{"name":"enum"}}],[0,"keyed","telemetry","",null,null],[3,"KeyedIgnoring","telemetry::keyed","",null,null],[3,"KeyedFlag","","",null,null],[3,"KeyedLinear","","",null,null],[3,"KeyedCount","","",null,null],[3,"KeyedEnum","","",null,null],[8,"KeyedHistogram","","A family of histograms, indexed by some dynamic value. Use these\nto monitor families of values that cannot be determined at\ncompile-time, e.g. add-ons, programs, etc.",null,null],[11,"record","","",7,{"inputs":[{"name":"keyedhistogram"},{"name":"k"},{"name":"t"}],"output":null}],[10,"record_cb","","",7,{"inputs":[{"name":"keyedhistogram"},{"name":"f"}],"output":null}],[11,"new","","",8,{"inputs":[{"name":"keyedignoring"}],"output":{"name":"keyedignoring"}}],[11,"record_cb","","",8,{"inputs":[{"name":"keyedignoring"},{"name":"f"}],"output":null}],[11,"new","","",9,{"inputs":[{"name":"keyedflag"},{"name":"service"},{"name":"string"}],"output":{"name":"keyedflag"}}],[11,"record_cb","","",9,{"inputs":[{"name":"keyedflag"},{"name":"f"}],"output":null}],[11,"new","","",10,{"inputs":[{"name":"keyedlinear"},{"name":"service"},{"name":"string"},{"name":"u32"},{"name":"u32"},{"name":"usize"}],"output":{"name":"keyedlinear"}}],[11,"record_cb","","",10,{"inputs":[{"name":"keyedlinear"},{"name":"f"}],"output":null}],[11,"record_cb","","",11,{"inputs":[{"name":"keyedcount"},{"name":"f"}],"output":null}],[11,"new","","",11,{"inputs":[{"name":"keyedcount"},{"name":"service"},{"name":"string"}],"output":{"name":"keyedcount"}}],[11,"record_cb","","",12,{"inputs":[{"name":"keyedenum"},{"name":"f"}],"output":null}],[11,"new","","",12,{"inputs":[{"name":"keyedenum"},{"name":"service"},{"name":"string"},{"name":"usize"}],"output":{"name":"keyedenum"}}],[11,"new","telemetry","",13,{"inputs":[{"name":"service"}],"output":{"name":"service"}}],[11,"to_json","","",13,{"inputs":[{"name":"service"},{"name":"serializationformat"},{"name":"sender"}],"output":null}],[11,"set_active","","",13,{"inputs":[{"name":"service"},{"name":"bool"}],"output":null}],[11,"is_active","","",13,{"inputs":[{"name":"service"}],"output":{"name":"bool"}}],[11,"drop","","Terminate the thread once the service is dead.",13,{"inputs":[{"name":"service"}],"output":null}],[6,"Version","","",null,null],[8,"Flatten","","",null,null],[10,"as_u32","","",14,{"inputs":[{"name":"flatten"}],"output":{"name":"u32"}}]],"paths":[[4,"SerializationFormat"],[8,"Histogram"],[3,"Ignoring"],[3,"Flag"],[3,"Linear"],[3,"Count"],[3,"Enum"],[8,"KeyedHistogram"],[3,"KeyedIgnoring"],[3,"KeyedFlag"],[3,"KeyedLinear"],[3,"KeyedCount"],[3,"KeyedEnum"],[3,"Service"],[8,"Flatten"]]};
initSearch(searchIndex);
