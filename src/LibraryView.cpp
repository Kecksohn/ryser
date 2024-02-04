#include "LibraryView.hh"

#include <iostream>
#include <ostream>
#include <QLabel>
#include <QVBoxLayout>

LibraryView::LibraryView(QWidget* parent) : QWidget(parent) {
    QVBoxLayout* layout = new QVBoxLayout(this);
    QLabel* label = new QLabel("This is the library view", this);
    label->setStyleSheet("QLabel { color : white; }");
    // Additional setup for the second view goes here
    label->setAlignment(Qt::AlignCenter);
    layout->addWidget(label);
}