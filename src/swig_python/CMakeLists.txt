if (CMAKE_CROSSCOMPILING)
    message("cross-compiling: skipping python bindings")
    return()
endif()
if(NOT SWIG_FOUND)
    message("skipping swig python, couldn't find swig")
    return()
endif()

if(NOT Python_FOUND)
    message("skipping swig python, couldn't find python")
    return()
endif()

find_program(SED REQUIRED NAMES gsed sed)

join_path(_swigCFile ${CMAKE_CURRENT_BINARY_DIR} "gasdk_py_wrap.c")
join_path(_hackyGDKInclude ${CMAKE_SOURCE_DIR} "include")

find_tool(_buildPyhonSwig "swig_python.sh")

add_custom_command(OUTPUT ${_swigCFile}
    COMMAND ${_buildPyhonSwig} ${SED} ${SWIG_EXECUTABLE} ${_swigCFile} ${CMAKE_CURRENT_BINARY_DIR} swig_gasdk.i python_extra.py_in ${_hackyGDKInclude}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS greenaddress-objects
)
add_custom_target(swig_python-cmd
    DEPENDS ${_swigCFile}
)

add_library(swig_python OBJECT ${_swigCFile})
get_target_property(_gdkIncludes greenaddress-objects INTERFACE_INCLUDE_DIRECTORIES)
target_include_directories(swig_python PRIVATE ${_gdkIncludes})
if(CMAKE_VERSION VERSION_LESS_EQUAL 3.12)
    target_include_directories(swig_python PRIVATE ${PYTHON_INCLUDE_DIRS})
else()
    target_link_libraries(swig_python PUBLIC Python::Python)
endif()

target_compile_options(swig_python
    PRIVATE 
        "-fasynchronous-unwind-tables"
        "-fexceptions"
        "-fstack-protector-strong"
        "-fno-strict-aliasing"
)
if(CMAKE_VERSION VERSION_GREATER 3.12)
    target_compile_options(swig_python
        PRIVATE 
            $<$<AND:$<CXX_COMPILER_ID:GNU,Clang>,$<PLATFORM_ID:Linux>>:
                "-Werror"
                "-Wall"
                "-Wextra"
                "-Wpedantic"
                "-Wshadow"
                ##### should be un-commented and errors should be fixed
                # "-Wconversion"
            >
    )
endif()
target_compile_definitions(swig_python PRIVATE _FORTIFY_SOURCE=2)

add_dependencies(swig_python-cmd greenaddress-objects)
add_dependencies(swig_python swig_python-cmd)
