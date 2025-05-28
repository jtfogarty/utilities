#!/bin/bash

# Exit on any error
set -e

# Variables
PYENV_VERSION="3.10.12"
VENV_NAME="jupyter_notebook_v3"
PYTHON_VERSION="3.10.12"
NOTEBOOK_VERSION="6.5.4"
JUPYTER_SERVER_VERSION="1.18.1"
JUPYTER_CLIENT_VERSION="7.4.9"
NBCLASSIC_VERSION="0.5.6"

# Check if pyenv is installed
if ! command -v pyenv &> /dev/null; then
    echo "Error: pyenv is not installed. Please install pyenv first."
    exit 1
fi

# Check if Python version is installed
if ! pyenv versions | grep -q "$PYTHON_VERSION"; then
    echo "Installing Python $PYTHON_VERSION..."
    pyenv install $PYTHON_VERSION
fi

# Check if virtual environment already exists
if pyenv virtualenvs | grep -q "$VENV_NAME"; then
    echo "Virtual environment '$VENV_NAME' already exists. Please choose a different name or delete the existing one."
    exit 1
fi

# Create a new virtual environment
echo "Creating virtual environment '$VENV_NAME' with Python $PYTHON_VERSION..."
pyenv virtualenv $PYTHON_VERSION $VENV_NAME

# Activate the virtual environment
echo "Activating virtual environment '$VENV_NAME'..."
eval "$(pyenv init -)"
pyenv activate $VENV_NAME

# Upgrade pip to the latest version
echo "Upgrading pip..."
pip install --upgrade pip

# Install specific versions of Jupyter Notebook, jupyter_server, jupyter_client, nbclassic, and jupyter_contrib_nbextensions
echo "Installing Jupyter Notebook (version $NOTEBOOK_VERSION), jupyter_server (version $JUPYTER_SERVER_VERSION), jupyter_client (version $JUPYTER_CLIENT_VERSION), nbclassic (version $NBCLASSIC_VERSION), and jupyter_contrib_nbextensions..."
pip install notebook==$NOTEBOOK_VERSION jupyter_server==$JUPYTER_SERVER_VERSION jupyter_client==$JUPYTER_CLIENT_VERSION nbclassic==$NBCLASSIC_VERSION jupyter_contrib_nbextensions

# Install the nbextensions
echo "Installing Jupyter contrib nbextensions..."
jupyter contrib nbextension install --user

# Enable the init_cell/main nbextension
echo "Enabling init_cell/main nbextension..."
jupyter nbextension enable init_cell/main

# Verify installation (with fallback if it fails)
echo "Verifying Jupyter Notebook installation..."
if jupyter notebook --version; then
    echo "Verification successful."
else
    echo "Verification failed, but the installation may still work. Test manually with 'jupyter notebook'."
fi

# Deactivate the virtual environment
echo "Deactivating virtual environment..."
pyenv deactivate

echo "Setup complete! To use Jupyter Notebook, activate the environment with:"
echo "pyenv activate $VENV_NAME"
echo "Then run 'jupyter notebook' to start the classic Notebook interface."
