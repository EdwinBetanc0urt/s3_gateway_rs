#!/usr/bin/env sh
# @autor Edwin Betancourt <EdwinBetanc0urt@outlook.com> https://github.com/EdwinBetanc0urt

PROD_FILE=/opt/apps/server/.env

# copy `template.env` file to `.env`
cp -rf /opt/apps/server/template.env $PROD_FILE

# Set server values
sed -i "s|info|$RUST_LOG|g" $PROD_FILE && \
sed -i "s|allowed_origin|$ALLOWED_ORIGIN|g" $PROD_FILE && \
sed -i "s|s3_url|$S3_URL|g" $PROD_FILE && \
sed -i "s|bucket_name|$BUCKET_NAME|g" $PROD_FILE && \
sed -i "s|api_key|$API_KEY|g" $PROD_FILE && \
sed -i "s|secret_key|$SECRET_KEY|g" $PROD_FILE

# Run app
server
