FROM python:3.13-slim@sha256:f2fdaec50160418e0c2867ba3e254755edd067171725886d5d303fd7057bbf81

WORKDIR /app

COPY backend.py .
COPY requirements.txt .

# Install dependencies
RUN pip install -r requirements.txt

EXPOSE 5317 5000

CMD ["python", "backend.py"]