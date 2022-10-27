#!/usr/bin/env bash
set -e

PYTHON_EXE=$1; shift
BUILD_ROOT=$1; shift
INSTALL_PREFIX=$1; shift

PYTHON_DESTDIR=${INSTALL_PREFIX}
mkdir -p $PYTHON_DESTDIR

VENV_DIR=${BUILD_ROOT}/venv
virtualenv --clear -p ${PYTHON_EXE} ${VENV_DIR}
source $VENV_DIR/bin/activate

cd $PYTHON_DESTDIR

cp -r ${BUILD_ROOT}/greenaddress .
cp ${BUILD_ROOT}/setup.py .

pip install -U pip setuptools wheel

pip wheel --wheel-dir=${PYTHON_DESTDIR} .
deactivate

virtualenv --clear -p ${PYTHON_EXE} ${BUILD_ROOT}/smoketestvenv
source ${BUILD_ROOT}/smoketestvenv/bin/activate

pip install --find-links=. greenaddress
python -c "import greenaddress; assert len(greenaddress.get_networks()) > 0"
deactivate
rm -fr ${BUILD_ROOT}/smoketestvenv
