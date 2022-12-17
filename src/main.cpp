/*
  Building this program happens outside of the cargo process.
  We simply need to link against the Rust library and the
  system libraries it depends upon

  g++ -std=c++17 -o cpp_program src/main.cpp \
      -I .. -I target/cxxbridge \
      -L target/debug -l tantivy_cpp_lib \
      -pthread -l dl
*/

// consider the ffi part of Rust code
#include "tantivy-cpp-lib/src/lib.rs.h"

#include <iostream>

int
main()
{
  std::cout << "starting from C++\n";
  rust_from_cpp();
  std::cout << "finishing with C++\n";

  rust::Box<Searcher> searcher = create_searcher("/tmp/searcher/");
  
  rust::Vec<DocumentField> document ;
  DocumentField title = DocumentField{"title","The Old Man and the Sea", FieldType::str_field};
  DocumentField body = DocumentField {"body", 
      "He was an old man who fished alone in a skiff in the Gulf Stream and \
         he had gone eighty-four days now without taking a fish.", FieldType::str_field};
  
  document.push_back(title);
  document.push_back(body);
  // document.push_back(DocumentField{"documId", "11", FieldType::long_field});

  rust::Vec<DocumentField> document2;

  DocumentField title2 = DocumentField { "title", "The Modern Prometheus", FieldType::str_field};
  DocumentField body2  = DocumentField{ "body", "You will rejoice to hear that no disaster has accompanied the commencement of an \
              enterprise which you have regarded with such evil forebodings.  I arrived here \
              yesterday, and my first task is to assure my dear sister of my welfare and \
              increasing confidence in the success of my undertaking." , FieldType::str_field};
  document2.push_back(title2);
  document2.push_back(body2);
  // document2.push_back(DocumentField{"documId", "12", FieldType::long_field});
  

  IdDocument idDocument1 = IdDocument {1001, document};
  IdDocument idDocument2 = IdDocument{2002, document2};

  rust::Vec<IdDocument> docs;

  docs.push_back(idDocument1);
  docs.push_back(idDocument2);

  add_document(*searcher, docs);

  rust::Vec<IdDocument> documents = search(*searcher, "sea evil");

  for(IdDocument doc : documents) {
    std::cout << "id:" <<  doc.docId  << " field_value:" << doc.fieldValues[0].field_value.c_str() << " " << doc.score << std::endl;
  } 

  return 0;
}
