FROM python:3.13-slim@sha256:079601253d5d25ae095110937ea8cfd7403917b53b077870bccd8b026dc7c42f

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]