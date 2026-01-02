#ifndef GRAPHVIEW_H
#define GRAPHVIEW_H

#include <QWidget>
#include <QGraphicsView>
#include <QGraphicsScene>
#include <QGraphicsEllipseItem>
#include <QGraphicsLineItem>
#include <QGraphicsTextItem>
#include <QString>
#include <QMap>

class GraphView : public QGraphicsView
{
    Q_OBJECT

public:
    explicit GraphView(QWidget *parent = nullptr);
    ~GraphView();

    void loadDotFile(const QString &filePath);
    void showPlaceholder(const QString &message);
    void clear();

protected:
    void wheelEvent(QWheelEvent *event) override;
    void drawBackground(QPainter *painter, const QRectF &rect) override;

private:
    void setupScene();
    void parseDotFile(const QString &content);
    void layoutGraph();
    QGraphicsEllipseItem* createNode(const QString &id, const QString &label);
    void createEdge(const QString &from, const QString &to);

    QGraphicsScene *m_scene;
    QMap<QString, QGraphicsEllipseItem*> m_nodes;
    QGraphicsTextItem *m_placeholderText;
    
    // Node styling
    static constexpr qreal NODE_WIDTH = 150;
    static constexpr qreal NODE_HEIGHT = 40;
    static constexpr qreal NODE_SPACING_X = 200;
    static constexpr qreal NODE_SPACING_Y = 80;
};

#endif // GRAPHVIEW_H
