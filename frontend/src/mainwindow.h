#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QToolBar>
#include <QStatusBar>
#include <QDockWidget>
#include <QListWidget>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QProcess>
#include <QSettings>

class GraphView;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    void selectFolder();
    void runAnalysis();
    void onAnalysisFinished(int exitCode, QProcess::ExitStatus status);
    void onAnalysisOutput();
    void clearResults();
    void showAbout();

private:
    void setupUI();
    void setupMenuBar();
    void setupToolBar();
    void setupSidebar();
    void setupCentralWidget();
    void setupStatusBar();
    void loadSettings();
    void saveSettings();
    void updateAnalyzeButton();

    // UI Components
    QToolBar *m_toolbar;
    QDockWidget *m_sidebarDock;
    QListWidget *m_fileList;
    QLineEdit *m_folderPath;
    QPushButton *m_browseBtn;
    QPushButton *m_analyzeBtn;
    QPushButton *m_clearBtn;
    QLabel *m_statusLabel;
    GraphView *m_graphView;

    // Backend process
    QProcess *m_analysisProcess;
    QString m_currentFolder;
    QString m_backendPath;
};

#endif // MAINWINDOW_H
