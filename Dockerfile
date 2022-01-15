FROM python:latest AS PY
WORKDIR /app
COPY requirements.txt .
RUN pip3 install -r ./requirements.txt
ENV FILE=./analysis.csv
COPY ./analysis.py .
CMD ["python3", "analysis.py"]



