/*
  Building this program happens outside of the cargo process.
  We simply need to link against the Rust library and the
  system libraries it depends upon

  g++ -std=c++17 -o cpp_program src/main.cpp \
      -I .. -I target/cxxbridge \
      -L target/debug -l arustlib \
      -pthread -l dl
*/

// consider the ffi part of Rust code
#include "arustlib/src/lib.rs.h"

#include <iostream>

int
main()
{
  std::cout << "starting from C++\n";
  rust_from_cpp();
  std::cout << "finishing with C++\n";

  rust::Box<Searcher> searcher = create_searcher("/tmp/searcher/");
  
  rust::Vec<DocumentField> document ;
  DocumentField title = DocumentField{"title","The Old Man and the Sea","String"};
  DocumentField body = DocumentField {"body", 
      "He was an old man who fished alone in a skiff in the Gulf Stream and \
         he had gone eighty-four days now without taking a fish.", "String"};
  
  document.push_back(title);
  document.push_back(body);

  add_document(*searcher, document);

  rust::Vec<IdDocument> documents = search(*searcher, "sea whale");

  for(IdDocument doc : documents) {
    std::cout << "id:" <<  doc.docId  << " title:" << doc.title.data() << " " << doc.score << std::endl;
  } 

  return 0;
}
