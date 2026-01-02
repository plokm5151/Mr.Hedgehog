#include "graphview.h"

#include <QFile>
#include <QTextStream>
#include <QWheelEvent>
#include <QPainter>
#include <QRegularExpression>
#include <QDebug>
#include <cmath>

GraphView::GraphView(QWidget *parent)
    : QGraphicsView(parent)
    , m_scene(nullptr)
    , m_placeholderText(nullptr)
{
    setupScene();
    
    // Enable smooth scrolling and rendering
    setRenderHint(QPainter::Antialiasing);
    setRenderHint(QPainter::TextAntialiasing);
    setRenderHint(QPainter::SmoothPixmapTransform);
    setViewportUpdateMode(QGraphicsView::FullViewportUpdate);
    setDragMode(QGraphicsView::ScrollHandDrag);
    setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
    
    // Background color
    setBackgroundBrush(QColor("#11111b"));
    
    // Frame style
    setFrameShape(QFrame::NoFrame);
    
    // Show initial placeholder
    showPlaceholder("Select a folder and click 'Run Analysis'\nto visualize the call graph");
}

GraphView::~GraphView()
{
    delete m_scene;
}

void GraphView::setupScene()
{
    m_scene = new QGraphicsScene(this);
    setScene(m_scene);
}

void GraphView::loadDotFile(const QString &filePath)
{
    QFile file(filePath);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        showPlaceholder("Failed to open output file:\n" + filePath);
        return;
    }
    
    QTextStream in(&file);
    QString content = in.readAll();
    file.close();
    
    parseDotFile(content);
}

void GraphView::parseDotFile(const QString &content)
{
    clear();
    
    // Parse DOT format
    // Nodes: "nodeid" [label="Node Label"];
    // Edges: "from" -> "to";
    
    QStringList lines = content.split('\n');
    QList<QPair<QString, QString>> edges;
    
    // First pass: extract nodes
    QRegularExpression nodeRegex("\"([^\"]+)\"\\s*\\[label=\"([^\"]+)\"\\]");
    QRegularExpression edgeRegex("\"([^\"]+)\"\\s*->\\s*\"([^\"]+)\"");
    
    for (const QString &line : lines) {
        // Check for node definition
        QRegularExpressionMatch nodeMatch = nodeRegex.match(line);
        if (nodeMatch.hasMatch()) {
            QString id = nodeMatch.captured(1);
            QString label = nodeMatch.captured(2);
            createNode(id, label);
            continue;
        }
        
        // Check for edge definition
        QRegularExpressionMatch edgeMatch = edgeRegex.match(line);
        if (edgeMatch.hasMatch()) {
            QString from = edgeMatch.captured(1);
            QString to = edgeMatch.captured(2);
            edges.append(qMakePair(from, to));
        }
    }
    
    // Layout nodes
    layoutGraph();
    
    // Create edges
    for (const auto &edge : edges) {
        createEdge(edge.first, edge.second);
    }
    
    // Fit view
    if (!m_nodes.isEmpty()) {
        setSceneRect(m_scene->itemsBoundingRect().adjusted(-50, -50, 50, 50));
        fitInView(m_scene->itemsBoundingRect(), Qt::KeepAspectRatio);
        scale(0.9, 0.9); // Slight zoom out
    } else {
        showPlaceholder("No nodes found in the call graph");
    }
}

QGraphicsEllipseItem* GraphView::createNode(const QString &id, const QString &label)
{
    if (m_nodes.contains(id)) {
        return m_nodes[id];
    }
    
    // Create rounded rectangle (using ellipse with large radius for rounded look)
    QGraphicsEllipseItem *node = m_scene->addEllipse(
        0, 0, NODE_WIDTH, NODE_HEIGHT,
        QPen(QColor("#89b4fa"), 2),
        QBrush(QColor("#313244"))
    );
    
    // Add label
    QString displayLabel = label;
    if (displayLabel.length() > 20) {
        displayLabel = displayLabel.right(20);
        displayLabel = "..." + displayLabel;
    }
    
    QGraphicsTextItem *text = m_scene->addText(displayLabel);
    text->setDefaultTextColor(QColor("#cdd6f4"));
    text->setParentItem(node);
    
    // Center text in node
    QRectF textRect = text->boundingRect();
    text->setPos(
        (NODE_WIDTH - textRect.width()) / 2,
        (NODE_HEIGHT - textRect.height()) / 2
    );
    
    m_nodes[id] = node;
    return node;
}

void GraphView::createEdge(const QString &from, const QString &to)
{
    if (!m_nodes.contains(from) || !m_nodes.contains(to)) {
        return;
    }
    
    QGraphicsEllipseItem *fromNode = m_nodes[from];
    QGraphicsEllipseItem *toNode = m_nodes[to];
    
    // Calculate center points
    QPointF fromCenter = fromNode->pos() + QPointF(NODE_WIDTH / 2, NODE_HEIGHT);
    QPointF toCenter = toNode->pos() + QPointF(NODE_WIDTH / 2, 0);
    
    // Create arrow line
    QGraphicsLineItem *line = m_scene->addLine(
        QLineF(fromCenter, toCenter),
        QPen(QColor("#a6adc8"), 1.5)
    );
    line->setZValue(-1); // Behind nodes
    
    // Add arrowhead
    qreal angle = std::atan2(toCenter.y() - fromCenter.y(), toCenter.x() - fromCenter.x());
    qreal arrowSize = 10;
    
    QPointF arrowP1 = toCenter - QPointF(
        std::cos(angle - M_PI / 6) * arrowSize,
        std::sin(angle - M_PI / 6) * arrowSize
    );
    QPointF arrowP2 = toCenter - QPointF(
        std::cos(angle + M_PI / 6) * arrowSize,
        std::sin(angle + M_PI / 6) * arrowSize
    );
    
    QPolygonF arrowHead;
    arrowHead << toCenter << arrowP1 << arrowP2;
    
    QGraphicsPolygonItem *arrow = m_scene->addPolygon(
        arrowHead,
        QPen(QColor("#a6adc8")),
        QBrush(QColor("#a6adc8"))
    );
    arrow->setZValue(-1);
}

void GraphView::layoutGraph()
{
    if (m_nodes.isEmpty()) return;
    
    // Simple hierarchical layout
    int row = 0;
    int col = 0;
    int maxCols = 5;
    
    for (auto it = m_nodes.begin(); it != m_nodes.end(); ++it) {
        it.value()->setPos(col * NODE_SPACING_X, row * NODE_SPACING_Y);
        
        col++;
        if (col >= maxCols) {
            col = 0;
            row++;
        }
    }
}

void GraphView::showPlaceholder(const QString &message)
{
    clear();
    
    m_placeholderText = m_scene->addText(message);
    m_placeholderText->setDefaultTextColor(QColor("#6c7086"));
    
    QFont font = m_placeholderText->font();
    font.setPointSize(16);
    m_placeholderText->setFont(font);
    
    // Center the text
    QRectF textRect = m_placeholderText->boundingRect();
    m_placeholderText->setPos(-textRect.width() / 2, -textRect.height() / 2);
    
    setSceneRect(m_scene->itemsBoundingRect().adjusted(-100, -100, 100, 100));
}

void GraphView::clear()
{
    m_scene->clear();
    m_nodes.clear();
    m_placeholderText = nullptr;
}

void GraphView::wheelEvent(QWheelEvent *event)
{
    // Zoom with mouse wheel
    const qreal scaleFactor = 1.1;
    
    if (event->angleDelta().y() > 0) {
        scale(scaleFactor, scaleFactor);
    } else {
        scale(1 / scaleFactor, 1 / scaleFactor);
    }
}

void GraphView::drawBackground(QPainter *painter, const QRectF &rect)
{
    QGraphicsView::drawBackground(painter, rect);
    
    // Draw subtle grid
    painter->setPen(QPen(QColor("#1e1e2e"), 0.5));
    
    qreal gridSize = 50;
    qreal left = int(rect.left()) - (int(rect.left()) % int(gridSize));
    qreal top = int(rect.top()) - (int(rect.top()) % int(gridSize));
    
    QVector<QLineF> lines;
    for (qreal x = left; x < rect.right(); x += gridSize) {
        lines.append(QLineF(x, rect.top(), x, rect.bottom()));
    }
    for (qreal y = top; y < rect.bottom(); y += gridSize) {
        lines.append(QLineF(rect.left(), y, rect.right(), y));
    }
    
    painter->drawLines(lines);
}
