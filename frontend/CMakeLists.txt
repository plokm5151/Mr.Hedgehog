cmake_minimum_required(VERSION 3.16)
project(TraceCraftUI VERSION 0.4.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find Qt packages
find_package(Qt6 REQUIRED COMPONENTS Core Widgets Gui)

# Source files
set(SOURCES
    src/main.cpp
    src/mainwindow.cpp
    src/graphview.cpp
)

set(HEADERS
    src/mainwindow.h
    src/graphview.h
)

set(RESOURCES
    resources/resources.qrc
)

# Create executable
add_executable(TraceCraftUI
    ${SOURCES}
    ${HEADERS}
    ${RESOURCES}
)

# Link Qt libraries
target_link_libraries(TraceCraftUI PRIVATE
    Qt6::Core
    Qt6::Widgets
    Qt6::Gui
)

# macOS specific settings
if(APPLE)
    set_target_properties(TraceCraftUI PROPERTIES
        MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_BUNDLE_NAME "TraceCraft"
        MACOSX_BUNDLE_BUNDLE_VERSION "${PROJECT_VERSION}"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_GUI_IDENTIFIER "com.tracecraft.ui"
        MACOSX_BUNDLE_ICON_FILE "AppIcon"
    )
    
    # Copy icon
    set(APP_ICON "${CMAKE_CURRENT_SOURCE_DIR}/../assets/icon.jpg")
    if(EXISTS ${APP_ICON})
        target_sources(TraceCraftUI PRIVATE ${APP_ICON})
        set_source_files_properties(${APP_ICON} PROPERTIES
            MACOSX_PACKAGE_LOCATION "Resources")
    endif()
endif()
