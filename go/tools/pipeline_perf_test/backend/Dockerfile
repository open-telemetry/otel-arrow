FROM python:3.11-slim

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]