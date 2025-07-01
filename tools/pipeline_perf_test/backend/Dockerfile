FROM python:3.13-slim@sha256:8b4a876684a35cd69714f5664c65b70eb6e3716617c7f3d1c84251df0c39ac58

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]