#include "HomeView.hh"

#include <QLabel>
#include <QVBoxLayout>

HomeView::HomeView(QWidget* parent) : QWidget(parent) {
    QVBoxLayout* layout = new QVBoxLayout(this);
    QLabel* label = new QLabel("This is the home view", this);
    label->setStyleSheet("QLabel { color : white; }");
    label->setAlignment(Qt::AlignCenter);
    layout->addWidget(label);
}
