#include "HomeView.hh"

#include <iostream>
#include <QLabel>
#include <QVBoxLayout>
#include <QKeyEvent>
#include <QPushButton>
#include <QDebug>

#include <boost/property_tree/json_parser.hpp>
#include "../manageconfig.hh"

HomeView::HomeView(QWidget* parent) : QWidget(parent) {

    setFocusPolicy(Qt::StrongFocus);

    QVBoxLayout* layout = new QVBoxLayout(this);

    boost::property_tree::ptree pt;
    if (!load_json(pt))
    {
	    QLabel* label = new QLabel("Failed to load config.json", this);
        layout->addWidget(label);
        return;
    }

    auto json_libraries = pt.get_child("Libraries");
    for (auto& library : json_libraries)
    {
	    QPushButton* button = new QPushButton(QString::fromStdString(library.first), this);
        button->setObjectName(QString::fromStdString(library.first));
		button->setFocusPolicy(Qt::StrongFocus);
		connect(button, &QPushButton::clicked, this, &HomeView::on_option_selected);
		layout->addWidget(button);
		libraries.emplace_back(button);
	}
	setLayout(layout);
}

void HomeView::keyPressEvent(QKeyEvent* event) {

    bool hasFocusedChild = false;
    for (QObject* child : children()) {
        QWidget* widget = qobject_cast<QWidget*>(child);
        if (widget && widget->hasFocus()) {
            hasFocusedChild = true;
            break;
        }
    }

    switch (event->key()) {
    case Qt::Key_Up:
        // Code to move focus to the previous widget
        if (!hasFocusedChild) {
            set_initial_focus();
            return;
        }
        focusPreviousChild();
        break;
    case Qt::Key_Down:
        // Code to move focus to the next widget
        if (!hasFocusedChild) {
            set_initial_focus();
            return;
        }
        focusNextChild();
        break;
    case Qt::Key_Right:
		// Go into the library view
		emit on_option_selected();
		break;
    default:
        QWidget::keyPressEvent(event);
    }
}

void HomeView::set_initial_focus() {
    for (QObject* child : children()) {
        QWidget* widget = qobject_cast<QWidget*>(child);
        if (widget && widget->focusPolicy() != Qt::NoFocus) {
            qDebug() << "Setting focus to:" << widget->objectName();
            widget->setFocus();
            break;
        }
    }
}

void HomeView::on_option_selected() {
    std::cout << "Option selected!\n";
}
