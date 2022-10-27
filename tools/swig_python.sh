#!/bin/bash
set -e

sed=$1; shift
swig=$1; shift
generated_c=$1; shift
outdir=$1; shift
swig_src=$1; shift
swig_extra=$1; shift
gdk_include=$1; shift

module_dir="${outdir}/greenaddress"
mkdir -p $module_dir

${swig} -python -py3 -threads -o ${generated_c} -I${gdk_include} -DGDK_API -outdir ${outdir} -o ${generated_c} ${swig_src}
swig_result="${outdir}/greenaddress.py"

# Fix up shared library names
${sed} -i 's/_greenaddress/libgreenaddress/g' ${generated_c} ${swig_result}

# Merge the extra helper code into greenaddress/__init__.py
mv ${swig_result} ${module_dir}/__init__.py
cat ${swig_extra} >>${module_dir}/__init__.py
