# Talky 

Talky is a rust application built upon tokio, axum and upon. It serves all files and folders from a directory recursively. It is possible to customize the design of the application by providing an index html file for each directory. Later, authentication, for instance in the form of a .htaccess file in the respective directories. Talky is heavily inspired by the CERN Indico plattform: https://indico.cern.ch. 


Use https://matze.github.io/axum-notes/notes/misc/serve_static_from_binary/index.html in order to bake the default index.html into the binary.