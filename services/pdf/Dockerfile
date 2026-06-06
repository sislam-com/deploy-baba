FROM public.ecr.aws/lambda/python:3.13

# WeasyPrint system dependencies
RUN dnf install -y \
    pango \
    pango-devel \
    cairo \
    cairo-devel \
    gdk-pixbuf2 \
    gdk-pixbuf2-devel \
    libffi-devel \
    && dnf clean all

COPY src/ ${LAMBDA_TASK_ROOT}/
COPY pyproject.toml ${LAMBDA_TASK_ROOT}/

RUN pip install --no-cache-dir weasyprint>=62 mangum>=0.19 fastapi>=0.115

CMD ["handler.handler"]
