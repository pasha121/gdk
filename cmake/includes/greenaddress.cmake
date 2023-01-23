include(GNUInstallDirs)
include(CMakePrintHelpers)



macro(create_greenaddress_target)
    add_library(greenaddress SHARED $<TARGET_OBJECTS:greenaddress-objects> $<TARGET_OBJECTS:sqlite3>)
    if(TARGET swig_java)
        target_sources(greenaddress PRIVATE $<TARGET_OBJECTS:swig_java>)
        target_link_libraries(greenaddress PRIVATE swig_java)
    endif()
    ### WARNING once on cmake > 3.12 ``target_sources(greenaddress-objects $<TARGET_NAME_IF_EXISTS:swig_python>)``
    if(TARGET swig_python)
        target_sources(greenaddress PRIVATE $<TARGET_OBJECTS:swig_python>)
     if(CMAKE_VERSION VERSION_LESS_EQUAL 3.12)
            target_link_libraries(greenaddress PRIVATE ${PYTHON_LIBRARIES})
        else()
            target_link_libraries(greenaddress PRIVATE swig_python)
        endif()
    endif()
    set_target_properties(greenaddress PROPERTIES
        VERSION ${PROJECT_VERSION}
        SOVERSION ${PROJECT_VERSION_MAJOR}
        PUBLIC_HEADER $<TARGET_PROPERTY:greenaddress-objects,PUBLIC_HEADER>
    )
    get_target_property(_gaIncludeDir greenaddress-objects INTERFACE_INCLUDE_DIRECTORIES)
    target_include_directories(greenaddress INTERFACE ${_gaIncludeDir})
    target_link_libraries(greenaddress PRIVATE
        gdk-rust
        Microsoft.GSL::GSL
        autobahn-cpp
        msgpackc-cxx
        websocketpp::websocketpp
        nlohmann_json::nlohmann_json
        tor
        PkgConfig::libevent
        PkgConfig::libsecp256k1
        $<$<NOT:$<PLATFORM_ID:Windows>>:PkgConfig::libevent_pthreads>
        Boost::boost
        Boost::log
        Boost::thread
        OpenSSL::SSL
        $<$<PLATFORM_ID:Android>:log>
        ZLIB::ZLIB
        $<$<NOT:$<PLATFORM_ID:Android>>:pthread>
    )
    get_target_property(_wallycoreLib PkgConfig::wallycore INTERFACE_LINK_LIBRARIES)
    #cmake 3.24 ==> $<LINK_LIBRARY:WHOLE_ARCHIVE,PkgConfig::wallycore>
    if(CMAKE_VERSION VERSION_LESS_EQUAL 3.12)
        set_target_properties(greenaddress PROPERTIES LINK_FLAGS "-Wl,--whole-archive ${_wallycoreLib} -Wl,--no-whole-archive")
    else()
        set(_gdkLinkOptions ${GDK_LINK_OPTIONS})
        if(APPLE)
            list(APPEND _gdkLinkOptions "-Wl,-force_load" "SHELL:${_wallycoreLib}")
        else()
            list(APPEND _gdkLinkOptions "LINKER:SHELL:--whole-archive" "SHELL:${_wallycoreLib}" "LINKER:SHELL:--no-whole-archive")
        endif()
        target_link_options(greenaddress PRIVATE "${_gdkLinkOptions}")
    endif()
    get_library_install_dir(_libInstallDir)
    get_cmake_install_dir(LIB_CMAKE_INSTALL_DIR ${_libInstallDir})
    install(TARGETS greenaddress
        EXPORT "greenaddress-target"
        RUNTIME EXCLUDE_FROM_ALL
        OBJECTS EXCLUDE_FROM_ALL
        ARCHIVE EXCLUDE_FROM_ALL
        LIBRARY DESTINATION ${_libInstallDir}
                COMPONENT gdk-runtime
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/gdk
                COMPONENT gdk-dev
                EXCLUDE_FROM_ALL
    )
    install(
        FILES
            ${wallycore_INCLUDE_DIRS}/wally_address.h
            ${wallycore_INCLUDE_DIRS}/wally_anti_exfil.h
            ${wallycore_INCLUDE_DIRS}/wally_bip32.h
            ${wallycore_INCLUDE_DIRS}/wally_bip38.h
            ${wallycore_INCLUDE_DIRS}/wally_bip39.h
            ${wallycore_INCLUDE_DIRS}/wally_core.h
            ${wallycore_INCLUDE_DIRS}/wally_crypto.h
            ${wallycore_INCLUDE_DIRS}/wally_elements.h
            ${wallycore_INCLUDE_DIRS}/wally_script.h
            ${wallycore_INCLUDE_DIRS}/wally_transaction.h
        COMPONENT gdk-dev
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/gdk/libwally-core/
        EXCLUDE_FROM_ALL
    )
    install(EXPORT "greenaddress-target"
        COMPONENT gdk-dev
        DESTINATION ${LIB_CMAKE_INSTALL_DIR}/cmake
        NAMESPACE ${PROJECT_NAME}::
        FILE "greenaddress-targets.cmake"
        EXCLUDE_FROM_ALL
    )
    find_program(OBJCOPY NAMES llvm-objcopy ${TOOLCHAIN_PREFIX}-objcopy objcopy HINTS ${ANDROID_TOOLCHAIN_ROOT})
    if(OBJCOPY)
        add_custom_command(OUTPUT libgreenaddress.syms
            COMMAND ${OBJCOPY} --only-keep-debug $<TARGET_FILE:greenaddress> libgreenaddress.syms
            DEPENDS greenaddress
            BYPRODUCTS libgreenaddress.syms
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        )
        add_custom_target(greenaddress-syms ALL
            DEPENDS libgreenaddress.syms
        )
        install(FILES ${CMAKE_BINARY_DIR}/libgreenaddress.syms
            DESTINATION ${_libInstallDir}
            COMPONENT gdk-runtime
        )
    endif()
endmacro()


macro(create_greenaddressstatic_target)
    add_library(greenaddress-static STATIC $<TARGET_OBJECTS:greenaddress-objects> $<TARGET_OBJECTS:sqlite3>)
    if(TARGET swig_java)
        target_sources(greenaddress-static PRIVATE $<TARGET_OBJECTS:swig_java>)
        target_link_libraries(greenaddress-static PRIVATE swig_java)
    endif()
    ### WARNING once on cmake > 3.12 ``target_sources(greenaddress-objects $<TARGET_NAME_IF_EXISTS:swig_python>)``
    if(TARGET swig_python)
        target_sources(greenaddress-static PRIVATE $<TARGET_OBJECTS:swig_python>)
        target_link_libraries(greenaddress-static PRIVATE swig_python)
    endif()
    get_target_property(_gaIncludeDir greenaddress-objects INTERFACE_INCLUDE_DIRECTORIES)
    target_include_directories(greenaddress-static INTERFACE ${_gaIncludeDir})
    target_link_libraries(greenaddress-static PUBLIC
        PkgConfig::wallycore
        PkgConfig::libsecp256k1
        gdk-rust
        Microsoft.GSL::GSL
        autobahn-cpp
        msgpackc-cxx
        websocketpp::websocketpp
        nlohmann_json::nlohmann_json
        tor
        PkgConfig::libevent
        $<$<NOT:$<PLATFORM_ID:Windows>>:PkgConfig::libevent_pthreads>
        Boost::boost
        Boost::log
        Boost::thread
        OpenSSL::SSL
        $<$<PLATFORM_ID:Android>:log>
        ZLIB::ZLIB
        $<$<NOT:$<PLATFORM_ID:Android>>:pthread>
    )
    target_link_options(greenaddress-static INTERFACE "${GDK_LINK_OPTIONS}")
    if(NOT CMAKE_VERSION VERSION_LESS_EQUAL 3.12)
        target_link_options(greenaddress-static PRIVATE "${GDK_LINK_OPTIONS}")
    endif()
    if (ADD_COVERAGE AND CMAKE_BUILD_TYPE STREQUAL Debug)
        target_link_options(greenaddress-static PUBLIC --coverage)
    endif()
endmacro()



macro(create_greenaddressfull_target)
    add_library(greenaddress-full STATIC $<TARGET_OBJECTS:greenaddress-objects> $<TARGET_OBJECTS:sqlite3>)
    set_target_properties(greenaddress-full PROPERTIES OUTPUT_NAME greenaddress_full)
    ### WARNING once on cmake > 3.12 ``target_sources(greenaddress-objects $<TARGET_NAME_IF_EXISTS:swig_java>)``
    if(TARGET swig_java)
        target_sources(greenaddress-full PRIVATE $<TARGET_OBJECTS:swig_java>)
    endif()
    ### WARNING once on cmake > 3.12 ``target_sources(greenaddress-objects $<TARGET_NAME_IF_EXISTS:swig_python>)``
    if(TARGET swig_python)
        target_sources(greenaddress-full PRIVATE $<TARGET_OBJECTS:swig_python>)
    endif()
    set_target_properties(greenaddress-full PROPERTIES
        VERSION ${PROJECT_VERSION}
        SOVERSION ${PROJECT_VERSION_MAJOR}
        PUBLIC_HEADER $<TARGET_PROPERTY:greenaddress-objects,PUBLIC_HEADER>
    )
    get_target_property(_gaIncludeDir greenaddress-objects INTERFACE_INCLUDE_DIRECTORIES)
    target_include_directories(greenaddress-full INTERFACE ${_gaIncludeDir})
    target_link_libraries(greenaddress-full INTERFACE
        $<$<NOT:$<PLATFORM_ID:Android>>:pthread>
        $<$<PLATFORM_ID:Linux>:dl>
    )
    set(_maybeLibeventPthreads "${libevent_LIBDIR}/libevent_pthreads.a")
    if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
        set(_maybeLibeventPthreads "")
    endif()
    unset(_torLibList)
    foreach(_torLib IN LISTS _torLibs)
        string(APPEND _torLibList "${_torLib} ")
    endforeach()
    configure_file(${CMAKE_SOURCE_DIR}/tools/archiver.sh.gen.in  archiver.sh.gen)
    file(GENERATE OUTPUT archiver.sh INPUT ${CMAKE_CURRENT_BINARY_DIR}/archiver.sh.gen)
    add_custom_command(TARGET greenaddress-full POST_BUILD
        COMMAND ./archiver.sh
    )
    if(NOT CMAKE_VERSION VERSION_LESS_EQUAL 3.12)
        target_link_options(greenaddress-full PRIVATE "${GDK_LINK_OPTIONS}")
    endif()
    add_dependencies(greenaddress-full gdk-rust)
    get_library_install_dir(_libInstallDir)
    get_cmake_install_dir(LIB_CMAKE_INSTALL_DIR ${_libInstallDir})
    install(TARGETS greenaddress-full
        EXPORT "greenaddress-full-target"
        RUNTIME EXCLUDE_FROM_ALL
        OBJECTS EXCLUDE_FROM_ALL
        LIBRARY EXCLUDE_FROM_ALL
        ARCHIVE DESTINATION ${_libInstallDir}
            COMPONENT gdk-dev
            EXCLUDE_FROM_ALL
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/gdk
            COMPONENT gdk-dev
            EXCLUDE_FROM_ALL
    )
    install(EXPORT "greenaddress-full-target"
        COMPONENT gdk-dev
        DESTINATION ${LIB_CMAKE_INSTALL_DIR}/cmake
        NAMESPACE ${PROJECT_NAME}::
        FILE "greenaddress-full-targets.cmake"
        EXCLUDE_FROM_ALL
    )
    install(
        FILES 
            ${wallycore_INCLUDE_DIRS}/wally_address.h
            ${wallycore_INCLUDE_DIRS}/wally_anti_exfil.h
            ${wallycore_INCLUDE_DIRS}/wally_bip32.h
            ${wallycore_INCLUDE_DIRS}/wally_bip38.h
            ${wallycore_INCLUDE_DIRS}/wally_bip39.h
            ${wallycore_INCLUDE_DIRS}/wally_core.h
            ${wallycore_INCLUDE_DIRS}/wally_crypto.h
            ${wallycore_INCLUDE_DIRS}/wally_elements.h
            ${wallycore_INCLUDE_DIRS}/wally_script.h
            ${wallycore_INCLUDE_DIRS}/wally_transaction.h
        COMPONENT gdk-dev
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/gdk/libwally-core/
        EXCLUDE_FROM_ALL
    )
endmacro()



