FROM python:3.11-slim@sha256:44f3da9d702a293500f4e23619db3bbd2558e5931a0b27e45792e54b1ea41933

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]