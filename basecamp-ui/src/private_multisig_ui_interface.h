#pragma once

#include <QObject>
#include <QString>
#include "interface.h"

class PrivateMultisigUiInterface : public PluginInterface
{
public:
    ~PrivateMultisigUiInterface() override = default;
};

#define PrivateMultisigUiInterface_iid "org.logos.PrivateMultisigUiInterface/0.1"
Q_DECLARE_INTERFACE(PrivateMultisigUiInterface, PrivateMultisigUiInterface_iid)
