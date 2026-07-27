cmake_minimum_required(VERSION 3.14)
project(PrivateMultisigUiPlugin LANGUAGES CXX)

if(DEFINED ENV{LOGOS_MODULE_BUILDER_ROOT})
    include($ENV{LOGOS_MODULE_BUILDER_ROOT}/cmake/LogosModule.cmake)
else()
    message(FATAL_ERROR "LogosModule.cmake not found. Set LOGOS_MODULE_BUILDER_ROOT.")
endif()

file(READ "${CMAKE_CURRENT_SOURCE_DIR}/metadata.json" METADATA_JSON)
string(JSON MODULE_NAME GET ${METADATA_JSON} name)
string(JSON MODULE_VERSION GET ${METADATA_JSON} version)
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/module_config.h"
    "#pragma once\n"
    "#define MODULE_NAME \"${MODULE_NAME}\"\n"
    "#define MODULE_VERSION \"${MODULE_VERSION}\"\n"
)
include_directories("${CMAKE_CURRENT_BINARY_DIR}")

logos_module(
    NAME ${MODULE_NAME}
    REP_FILE src/private_multisig_ui.rep
    SOURCES
        src/private_multisig_ui_interface.h
        src/private_multisig_ui_plugin.h
        src/private_multisig_ui_plugin.cpp
)
