#include <QApplication>
#include "mainwindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    
    // Application metadata
    QApplication::setApplicationName("TraceCraft");
    QApplication::setApplicationVersion("0.4.0");
    QApplication::setOrganizationName("TraceCraft");
    
    MainWindow window;
    window.show();
    
    return app.exec();
}
