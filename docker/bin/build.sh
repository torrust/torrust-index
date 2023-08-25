#!/bin/bash

docker build --target debug --tag torrust-index-backend:debug .
docker build --target debug --tag torrust-tracker:debug https://github.com/torrust/torrust-tracker.git#pull/363/head	
