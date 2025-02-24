use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::Result;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::TypedBlock;
use futuresdr::runtime::WorkIo;

/// Push samples into [Zenoh](https://zenoh.io/) socket.
pub struct PubSink<T: Send + 'static> {
    config: zenoh::Config,
    key_expression: String,
    session: Option<zenoh::Session>,
    publisher: Option<zenoh::pubsub::Publisher<'static>>,
    _type: std::marker::PhantomData<T>,
    min_item: usize,
}

impl<T: Send + 'static> PubSink<T> {
    /// Create PubSink
    pub fn new(
        config: zenoh::Config,
        key_expression: impl Into<String>,
        min_item: usize,
    ) -> TypedBlock<Self> {
        TypedBlock::new(
            BlockMetaBuilder::new("PubSink").blocking().build(),
            StreamIoBuilder::new().add_input::<T>("in").build(),
            MessageIoBuilder::new().build(),
            PubSink {
                config: config,
                key_expression: key_expression.into(),
                session: None,
                publisher: None,
                _type: std::marker::PhantomData::<T>,
                min_item,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<T: Send + 'static> Kernel for PubSink<T> {
    async fn work(
        &mut self,
        work_io: &mut WorkIo,
        stream_io: &mut StreamIo,
        _message_io: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let input = stream_io.input(0).slice::<T>();

        let input_len = input.len();

        if input_len > 0 && input_len > self.min_item {
            let input = stream_io.input(0).slice_unchecked::<u8>();

            self.publisher.as_mut().unwrap().put(input).await.unwrap();

            stream_io.input(0).consume(input_len);
        }

        if stream_io.input(0).finished() {
            work_io.finished = true;
        }

        Ok(())
    }

    async fn init(
        &mut self,
        _stream_io: &mut StreamIo,
        _message_io: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let session = zenoh::open(self.config.clone()).await.unwrap();
        let publisher = session
            .declare_publisher(self.key_expression.clone())
            .await
            .unwrap();

        self.session = Some(session);
        self.publisher = Some(publisher);

        Ok(())
    }
}

/// Build a Zenoh [PubSink].
pub struct PubSinkBuilder<T: Send + 'static> {
    config: zenoh::Config,
    key_expression: String,
    _type: std::marker::PhantomData<T>,
    /// Minimum number of items per send
    min_item: usize,
}

impl<T: Send + 'static> PubSinkBuilder<T> {
    /// Create PubSink builder
    pub fn new() -> PubSinkBuilder<T> {
        PubSinkBuilder {
            config: zenoh::Config::default(),
            key_expression: "future-sdr/*".into(),
            _type: std::marker::PhantomData,
            min_item: 1,
        }
    }

    /// Zenoh configuration
    #[must_use]
    pub fn config(mut self, config: zenoh::Config) -> PubSinkBuilder<T> {
        self.config = config;
        self
    }

    /// Zenoh key expression
    #[must_use]
    pub fn key_expression(mut self, key_expression: &str) -> PubSinkBuilder<T> {
        self.key_expression = key_expression.to_string();
        self
    }

    /// Set minimum number of items in send buffer
    pub fn min_item_per_send(mut self, min_item: usize) -> PubSinkBuilder<T> {
        self.min_item = min_item;
        self
    }

    /// Build PubSink
    pub fn build(self) -> TypedBlock<PubSink<T>> {
        PubSink::<T>::new(self.config, self.key_expression, self.min_item)
    }
}

impl<T: Send + 'static> Default for PubSinkBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
