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

  rust::Vec<FieldMapping> field_mappings;
  field_mappings.push_back(FieldMapping{"title", FieldType::text_field});
  field_mappings.push_back(FieldMapping{"body",  FieldType::text_field});

  rust::Box<Searcher> searcher = create_searcher("/tmp/searcher/", field_mappings);
  
  rust::Vec<DocumentField> document ;
  DocumentField title = DocumentField{"title","The Old Man and the Sea", FieldType::text_field};
  DocumentField body = DocumentField {"body", 
      "He was an old man who fished alone in a skiff in the Gulf Stream and \
         he had gone eighty-four days now without taking a fish.", FieldType::text_field};
  
  document.push_back(title);
  document.push_back(body);
  // document.push_back(DocumentField{"documId", "11", FieldType::long_field});

  rust::Vec<DocumentField> document2;

  DocumentField title2 = DocumentField { "title", "The Modern Prometheus", FieldType::text_field};
  DocumentField body2  = DocumentField{ "body", "You will rejoice to hear that no disaster has accompanied the commencement of an \
              enterprise which you have regarded with such evil forebodings.  I arrived here \
              yesterday, and my first task is to assure my dear sister of my welfare and \
              increasing confidence in the success of my undertaking." , FieldType::text_field};
  document2.push_back(title2);
  document2.push_back(body2);
  // document2.push_back(DocumentField{"documId", "12", FieldType::long_field});
  
  rust::Vec<DocumentField> document3;
  DocumentField title3 = DocumentField { "title", "Scientific Computing", FieldType::text_field};
  DocumentField body3  = DocumentField{ "body", "Heath 2/e, presents a broad overview of numerical \
      methods for solving all the major problems in scientific computing,  \
      including linear and nonlinearequations, least squares, eigenvalues, \
      optimization, interpolation, integration, ordinary and partial differential equations, \
      fast Fourier transforms, and random number generators. The treatment is comprehensive yet concise, software" , 
    FieldType::text_field};
  document3.push_back(title3);
  document3.push_back(body3);

  rust::Vec<IdDocument> docs;

  docs.push_back(IdDocument {1001, document});
  docs.push_back(IdDocument{2002, document2});
  docs.push_back(IdDocument{2003, document3});

  add_document(*searcher, docs);

  rust::Vec<IdDocument> documents = search(*searcher, "sea task");

  for(IdDocument doc : documents) {
    std::cout << "id:" <<  doc.docId  << " field_value:" << doc.fieldValues[0].field_value.c_str() << " " << doc.score << std::endl;
  } 

  return 0;
}
