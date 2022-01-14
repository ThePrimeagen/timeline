FROM python:latest AS PY
WORKDIR /app
COPY requirements.txt .
RUN pip3 install -r ./requirements.txt
COPY ./out2.csv .
ENV FILE=./out2.csv
COPY ./analysis.py .
CMD ["python3", "analysis.py"]



