FROM python:3.13-slim@sha256:0889259e637f520dc0c462930dcde2bbd1e204582ae1824bfe51835064bf8036

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]