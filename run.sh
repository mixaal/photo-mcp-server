#!/bin/bash -x

YOLO_HOME="$HOME/git/YOLOv8-rs"
export YOLO_V8_MODEL_PATH="$YOLO_HOME/models"
export LIBTORCH_LIB="$YOLO_HOME/libtorch/lib"
export LD_LIBRARY_PATH=$LIBTORCH_LIB
export LIBTORCH="$YOLO_HOME/libtorch/"
export IMAGE_DIR="$HOME/Fotky/"
target/release/photo-mcp-server
