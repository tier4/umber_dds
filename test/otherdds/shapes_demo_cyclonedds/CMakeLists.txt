#
# Copyright(c) 2019 to 2022 ZettaScale Technology and others
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License v. 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Eclipse Distribution License
# v. 1.0 which is available at
# http://www.eclipse.org/org/documents/edl-v10.php.
#
# SPDX-License-Identifier: EPL-2.0 OR BSD-3-Clause
#
cmake_minimum_required(VERSION 3.16)
project(cycloneshapesdemo LANGUAGES C)

if(NOT TARGET CycloneDDS::ddsc)
  # Find the CycloneDDS package.
  find_package(CycloneDDS REQUIRED)
endif()

# This is a convenience function, provided by the CycloneDDS package,
# that will supply a library target related the the given idl file.
# In short, it takes the idl file, generates the source files with
# the proper data types and compiles them into a library.
idlc_generate(TARGET ShapeType_lib FILES "Shape.idl" WARNINGS no-implicit-extensibility)

# Both executables have only one related source file.
add_executable(ShapesDemoPublisher publisher.c)
add_executable(ShapesDemoSubscriber subscriber.c)

# Both executables need to be linked to the idl data type library and
# the ddsc API library.
target_link_libraries(ShapesDemoPublisher ShapeType_lib CycloneDDS::ddsc)
target_link_libraries(ShapesDemoSubscriber ShapeType_lib CycloneDDS::ddsc)
