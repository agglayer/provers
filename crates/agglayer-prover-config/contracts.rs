#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path, time::Duration,
};
use prover_config::{default_max_concurrency_limit, NetworkProverConfig, ProverType};
use prover_logger::log::Log;
use prover_utils::with;
use serde::{Deserialize, Serialize};
pub use crate::{shutdown::ShutdownConfig, telemetry::TelemetryConfig};
pub mod shutdown {
    use std::time::Duration;
    use serde::{Deserialize, Serialize};
    #[serde(rename_all = "kebab-case")]
    pub struct ShutdownConfig {
        #[serde(default = "default_shutdown_runtime_timeout")]
        #[serde(with = "crate::with::HumanDuration")]
        pub runtime_timeout: Duration,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ShutdownConfig {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "ShutdownConfig",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "runtime-timeout",
                    {
                        #[doc(hidden)]
                        struct __SerializeWith<'__a> {
                            values: (&'__a Duration,),
                            phantom: _serde::__private::PhantomData<ShutdownConfig>,
                        }
                        #[automatically_derived]
                        impl<'__a> _serde::Serialize for __SerializeWith<'__a> {
                            fn serialize<__S>(
                                &self,
                                __s: __S,
                            ) -> _serde::__private::Result<__S::Ok, __S::Error>
                            where
                                __S: _serde::Serializer,
                            {
                                crate::with::HumanDuration::serialize(self.values.0, __s)
                            }
                        }
                        &__SerializeWith {
                            values: (&self.runtime_timeout,),
                            phantom: _serde::__private::PhantomData::<ShutdownConfig>,
                        }
                    },
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ShutdownConfig {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "runtime-timeout" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"runtime-timeout" => {
                                _serde::__private::Ok(__Field::__field0)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<ShutdownConfig>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ShutdownConfig;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct ShutdownConfig",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match {
                            #[doc(hidden)]
                            struct __DeserializeWith<'de> {
                                value: Duration,
                                phantom: _serde::__private::PhantomData<ShutdownConfig>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            #[automatically_derived]
                            impl<'de> _serde::Deserialize<'de>
                            for __DeserializeWith<'de> {
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::__private::Ok(__DeserializeWith {
                                        value: crate::with::HumanDuration::deserialize(
                                            __deserializer,
                                        )?,
                                        phantom: _serde::__private::PhantomData,
                                        lifetime: _serde::__private::PhantomData,
                                    })
                                }
                            }
                            _serde::__private::Option::map(
                                _serde::de::SeqAccess::next_element::<
                                    __DeserializeWith<'de>,
                                >(&mut __seq)?,
                                |__wrap| __wrap.value,
                            )
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => default_shutdown_runtime_timeout(),
                        };
                        _serde::__private::Ok(ShutdownConfig {
                            runtime_timeout: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Duration> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "runtime-timeout",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some({
                                        #[doc(hidden)]
                                        struct __DeserializeWith<'de> {
                                            value: Duration,
                                            phantom: _serde::__private::PhantomData<ShutdownConfig>,
                                            lifetime: _serde::__private::PhantomData<&'de ()>,
                                        }
                                        #[automatically_derived]
                                        impl<'de> _serde::Deserialize<'de>
                                        for __DeserializeWith<'de> {
                                            fn deserialize<__D>(
                                                __deserializer: __D,
                                            ) -> _serde::__private::Result<Self, __D::Error>
                                            where
                                                __D: _serde::Deserializer<'de>,
                                            {
                                                _serde::__private::Ok(__DeserializeWith {
                                                    value: crate::with::HumanDuration::deserialize(
                                                        __deserializer,
                                                    )?,
                                                    phantom: _serde::__private::PhantomData,
                                                    lifetime: _serde::__private::PhantomData,
                                                })
                                            }
                                        }
                                        match _serde::de::MapAccess::next_value::<
                                            __DeserializeWith<'de>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__wrapper) => __wrapper.value,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    });
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => default_shutdown_runtime_timeout(),
                        };
                        _serde::__private::Ok(ShutdownConfig {
                            runtime_timeout: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["runtime-timeout"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "ShutdownConfig",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<ShutdownConfig>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for ShutdownConfig {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ShutdownConfig",
                "runtime_timeout",
                &&self.runtime_timeout,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ShutdownConfig {
        #[inline]
        fn clone(&self) -> ShutdownConfig {
            let _: ::core::clone::AssertParamIsClone<Duration>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ShutdownConfig {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ShutdownConfig {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ShutdownConfig {
        #[inline]
        fn eq(&self, other: &ShutdownConfig) -> bool {
            self.runtime_timeout == other.runtime_timeout
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ShutdownConfig {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Duration>;
        }
    }
    impl Default for ShutdownConfig {
        fn default() -> Self {
            Self {
                runtime_timeout: default_shutdown_runtime_timeout(),
            }
        }
    }
    const fn default_shutdown_runtime_timeout() -> Duration {
        Duration::from_secs(5)
    }
}
pub(crate) mod telemetry {
    use std::net::SocketAddr;
    use serde::{Deserialize, Serialize};
    use super::DEFAULT_IP;
    #[serde(rename_all = "kebab-case")]
    pub struct TelemetryConfig {
        #[serde(
            rename = "prometheus-addr",
            alias = "PrometheusAddr",
            default = "default_metrics_api_addr"
        )]
        pub addr: SocketAddr,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for TelemetryConfig {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "TelemetryConfig",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "prometheus-addr",
                    &self.addr,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for TelemetryConfig {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "PrometheusAddr" => _serde::__private::Ok(__Field::__field0),
                            "prometheus-addr" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"PrometheusAddr" => _serde::__private::Ok(__Field::__field0),
                            b"prometheus-addr" => {
                                _serde::__private::Ok(__Field::__field0)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<TelemetryConfig>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = TelemetryConfig;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct TelemetryConfig",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            SocketAddr,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => default_metrics_api_addr(),
                        };
                        _serde::__private::Ok(TelemetryConfig { addr: __field0 })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<SocketAddr> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "prometheus-addr",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<SocketAddr>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => default_metrics_api_addr(),
                        };
                        _serde::__private::Ok(TelemetryConfig { addr: __field0 })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "PrometheusAddr",
                    "prometheus-addr",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "TelemetryConfig",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<TelemetryConfig>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for TelemetryConfig {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "TelemetryConfig",
                "addr",
                &&self.addr,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TelemetryConfig {
        #[inline]
        fn clone(&self) -> TelemetryConfig {
            let _: ::core::clone::AssertParamIsClone<SocketAddr>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for TelemetryConfig {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for TelemetryConfig {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for TelemetryConfig {
        #[inline]
        fn eq(&self, other: &TelemetryConfig) -> bool {
            self.addr == other.addr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for TelemetryConfig {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<SocketAddr>;
        }
    }
    impl Default for TelemetryConfig {
        fn default() -> Self {
            Self {
                addr: default_metrics_api_addr(),
            }
        }
    }
    const fn default_metrics_api_addr() -> SocketAddr {
        SocketAddr::V4(std::net::SocketAddrV4::new(DEFAULT_IP, 3000))
    }
}
pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);
/// The Agglayer Prover configuration.
#[serde(rename_all = "kebab-case")]
pub struct ProverConfig {
    /// The gRPC endpoint used by the prover.
    #[serde(default = "default_socket_addr")]
    pub grpc_endpoint: SocketAddr,
    #[serde(default, skip_serializing_if = "crate::default")]
    pub grpc: GrpcConfig,
    /// The log configuration.
    #[serde(default, alias = "Log")]
    pub log: Log,
    /// Telemetry configuration.
    #[serde(default, alias = "Telemetry")]
    pub telemetry: TelemetryConfig,
    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,
    /// The maximum number of concurrent queries the prover can handle.
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,
    /// The maximum duration of a request.
    #[serde(default = "default_max_request_duration")]
    #[serde(with = "crate::with::HumanDuration")]
    pub max_request_duration: Duration,
    /// The maximum number of buffered queries.
    #[serde(default = "default_max_buffered_queries")]
    pub max_buffered_queries: usize,
    /// The primary prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub primary_prover: ProverType,
    /// The fallback prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub fallback_prover: Option<ProverType>,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ProverConfig {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "ProverConfig",
                false as usize + 1 + if crate::default(&self.grpc) { 0 } else { 1 } + 1
                    + 1 + 1 + 1 + 1 + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "grpc-endpoint",
                &self.grpc_endpoint,
            )?;
            if !crate::default(&self.grpc) {
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "grpc",
                    &self.grpc,
                )?;
            } else {
                _serde::ser::SerializeStruct::skip_field(&mut __serde_state, "grpc")?;
            }
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "log",
                &self.log,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "telemetry",
                &self.telemetry,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "shutdown",
                &self.shutdown,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "max-concurrency-limit",
                &self.max_concurrency_limit,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "max-request-duration",
                {
                    #[doc(hidden)]
                    struct __SerializeWith<'__a> {
                        values: (&'__a Duration,),
                        phantom: _serde::__private::PhantomData<ProverConfig>,
                    }
                    #[automatically_derived]
                    impl<'__a> _serde::Serialize for __SerializeWith<'__a> {
                        fn serialize<__S>(
                            &self,
                            __s: __S,
                        ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                        {
                            crate::with::HumanDuration::serialize(self.values.0, __s)
                        }
                    }
                    &__SerializeWith {
                        values: (&self.max_request_duration,),
                        phantom: _serde::__private::PhantomData::<ProverConfig>,
                    }
                },
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "max-buffered-queries",
                &self.max_buffered_queries,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "primary-prover",
                &self.primary_prover,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "fallback-prover",
                &self.fallback_prover,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for ProverConfig {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __field5,
                __field6,
                __field7,
                __field8,
                __field9,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        4u64 => _serde::__private::Ok(__Field::__field4),
                        5u64 => _serde::__private::Ok(__Field::__field5),
                        6u64 => _serde::__private::Ok(__Field::__field6),
                        7u64 => _serde::__private::Ok(__Field::__field7),
                        8u64 => _serde::__private::Ok(__Field::__field8),
                        9u64 => _serde::__private::Ok(__Field::__field9),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "grpc-endpoint" => _serde::__private::Ok(__Field::__field0),
                        "grpc" => _serde::__private::Ok(__Field::__field1),
                        "Log" => _serde::__private::Ok(__Field::__field2),
                        "log" => _serde::__private::Ok(__Field::__field2),
                        "Telemetry" => _serde::__private::Ok(__Field::__field3),
                        "telemetry" => _serde::__private::Ok(__Field::__field3),
                        "shutdown" => _serde::__private::Ok(__Field::__field4),
                        "max-concurrency-limit" => {
                            _serde::__private::Ok(__Field::__field5)
                        }
                        "max-request-duration" => {
                            _serde::__private::Ok(__Field::__field6)
                        }
                        "max-buffered-queries" => {
                            _serde::__private::Ok(__Field::__field7)
                        }
                        "primary-prover" => _serde::__private::Ok(__Field::__field8),
                        "fallback-prover" => _serde::__private::Ok(__Field::__field9),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"grpc-endpoint" => _serde::__private::Ok(__Field::__field0),
                        b"grpc" => _serde::__private::Ok(__Field::__field1),
                        b"Log" => _serde::__private::Ok(__Field::__field2),
                        b"log" => _serde::__private::Ok(__Field::__field2),
                        b"Telemetry" => _serde::__private::Ok(__Field::__field3),
                        b"telemetry" => _serde::__private::Ok(__Field::__field3),
                        b"shutdown" => _serde::__private::Ok(__Field::__field4),
                        b"max-concurrency-limit" => {
                            _serde::__private::Ok(__Field::__field5)
                        }
                        b"max-request-duration" => {
                            _serde::__private::Ok(__Field::__field6)
                        }
                        b"max-buffered-queries" => {
                            _serde::__private::Ok(__Field::__field7)
                        }
                        b"primary-prover" => _serde::__private::Ok(__Field::__field8),
                        b"fallback-prover" => _serde::__private::Ok(__Field::__field9),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<ProverConfig>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = ProverConfig;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct ProverConfig",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        SocketAddr,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_socket_addr(),
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        GrpcConfig,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<
                        Log,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<
                        TelemetryConfig,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field4 = match _serde::de::SeqAccess::next_element::<
                        ShutdownConfig,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field5 = match _serde::de::SeqAccess::next_element::<
                        usize,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_max_concurrency_limit(),
                    };
                    let __field6 = match {
                        #[doc(hidden)]
                        struct __DeserializeWith<'de> {
                            value: Duration,
                            phantom: _serde::__private::PhantomData<ProverConfig>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __DeserializeWith<'de> {
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                            {
                                _serde::__private::Ok(__DeserializeWith {
                                    value: crate::with::HumanDuration::deserialize(
                                        __deserializer,
                                    )?,
                                    phantom: _serde::__private::PhantomData,
                                    lifetime: _serde::__private::PhantomData,
                                })
                            }
                        }
                        _serde::__private::Option::map(
                            _serde::de::SeqAccess::next_element::<
                                __DeserializeWith<'de>,
                            >(&mut __seq)?,
                            |__wrap| __wrap.value,
                        )
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_max_request_duration(),
                    };
                    let __field7 = match _serde::de::SeqAccess::next_element::<
                        usize,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_max_buffered_queries(),
                    };
                    let __field8 = match _serde::de::SeqAccess::next_element::<
                        ProverType,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field9 = match _serde::de::SeqAccess::next_element::<
                        Option<ProverType>,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    _serde::__private::Ok(ProverConfig {
                        grpc_endpoint: __field0,
                        grpc: __field1,
                        log: __field2,
                        telemetry: __field3,
                        shutdown: __field4,
                        max_concurrency_limit: __field5,
                        max_request_duration: __field6,
                        max_buffered_queries: __field7,
                        primary_prover: __field8,
                        fallback_prover: __field9,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<SocketAddr> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<GrpcConfig> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<Log> = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<TelemetryConfig> = _serde::__private::None;
                    let mut __field4: _serde::__private::Option<ShutdownConfig> = _serde::__private::None;
                    let mut __field5: _serde::__private::Option<usize> = _serde::__private::None;
                    let mut __field6: _serde::__private::Option<Duration> = _serde::__private::None;
                    let mut __field7: _serde::__private::Option<usize> = _serde::__private::None;
                    let mut __field8: _serde::__private::Option<ProverType> = _serde::__private::None;
                    let mut __field9: _serde::__private::Option<Option<ProverType>> = _serde::__private::None;
                    while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "grpc-endpoint",
                                        ),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<SocketAddr>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("grpc"),
                                    );
                                }
                                __field1 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<GrpcConfig>(&mut __map)?,
                                );
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("log"),
                                    );
                                }
                                __field2 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<Log>(&mut __map)?,
                                );
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "telemetry",
                                        ),
                                    );
                                }
                                __field3 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<
                                        TelemetryConfig,
                                    >(&mut __map)?,
                                );
                            }
                            __Field::__field4 => {
                                if _serde::__private::Option::is_some(&__field4) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "shutdown",
                                        ),
                                    );
                                }
                                __field4 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<
                                        ShutdownConfig,
                                    >(&mut __map)?,
                                );
                            }
                            __Field::__field5 => {
                                if _serde::__private::Option::is_some(&__field5) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max-concurrency-limit",
                                        ),
                                    );
                                }
                                __field5 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                );
                            }
                            __Field::__field6 => {
                                if _serde::__private::Option::is_some(&__field6) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max-request-duration",
                                        ),
                                    );
                                }
                                __field6 = _serde::__private::Some({
                                    #[doc(hidden)]
                                    struct __DeserializeWith<'de> {
                                        value: Duration,
                                        phantom: _serde::__private::PhantomData<ProverConfig>,
                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                    }
                                    #[automatically_derived]
                                    impl<'de> _serde::Deserialize<'de>
                                    for __DeserializeWith<'de> {
                                        fn deserialize<__D>(
                                            __deserializer: __D,
                                        ) -> _serde::__private::Result<Self, __D::Error>
                                        where
                                            __D: _serde::Deserializer<'de>,
                                        {
                                            _serde::__private::Ok(__DeserializeWith {
                                                value: crate::with::HumanDuration::deserialize(
                                                    __deserializer,
                                                )?,
                                                phantom: _serde::__private::PhantomData,
                                                lifetime: _serde::__private::PhantomData,
                                            })
                                        }
                                    }
                                    match _serde::de::MapAccess::next_value::<
                                        __DeserializeWith<'de>,
                                    >(&mut __map) {
                                        _serde::__private::Ok(__wrapper) => __wrapper.value,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                });
                            }
                            __Field::__field7 => {
                                if _serde::__private::Option::is_some(&__field7) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max-buffered-queries",
                                        ),
                                    );
                                }
                                __field7 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                );
                            }
                            __Field::__field8 => {
                                if _serde::__private::Option::is_some(&__field8) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "primary-prover",
                                        ),
                                    );
                                }
                                __field8 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<ProverType>(&mut __map)?,
                                );
                            }
                            __Field::__field9 => {
                                if _serde::__private::Option::is_some(&__field9) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "fallback-prover",
                                        ),
                                    );
                                }
                                __field9 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<
                                        Option<ProverType>,
                                    >(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => default_socket_addr(),
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field4 = match __field4 {
                        _serde::__private::Some(__field4) => __field4,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field5 = match __field5 {
                        _serde::__private::Some(__field5) => __field5,
                        _serde::__private::None => default_max_concurrency_limit(),
                    };
                    let __field6 = match __field6 {
                        _serde::__private::Some(__field6) => __field6,
                        _serde::__private::None => default_max_request_duration(),
                    };
                    let __field7 = match __field7 {
                        _serde::__private::Some(__field7) => __field7,
                        _serde::__private::None => default_max_buffered_queries(),
                    };
                    let __field8 = match __field8 {
                        _serde::__private::Some(__field8) => __field8,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    let __field9 = match __field9 {
                        _serde::__private::Some(__field9) => __field9,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    _serde::__private::Ok(ProverConfig {
                        grpc_endpoint: __field0,
                        grpc: __field1,
                        log: __field2,
                        telemetry: __field3,
                        shutdown: __field4,
                        max_concurrency_limit: __field5,
                        max_request_duration: __field6,
                        max_buffered_queries: __field7,
                        primary_prover: __field8,
                        fallback_prover: __field9,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &[
                "grpc-endpoint",
                "grpc",
                "Log",
                "log",
                "Telemetry",
                "telemetry",
                "shutdown",
                "max-concurrency-limit",
                "max-request-duration",
                "max-buffered-queries",
                "primary-prover",
                "fallback-prover",
            ];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "ProverConfig",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<ProverConfig>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for ProverConfig {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "grpc_endpoint",
            "grpc",
            "log",
            "telemetry",
            "shutdown",
            "max_concurrency_limit",
            "max_request_duration",
            "max_buffered_queries",
            "primary_prover",
            "fallback_prover",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.grpc_endpoint,
            &self.grpc,
            &self.log,
            &self.telemetry,
            &self.shutdown,
            &self.max_concurrency_limit,
            &self.max_request_duration,
            &self.max_buffered_queries,
            &self.primary_prover,
            &&self.fallback_prover,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "ProverConfig",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ProverConfig {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ProverConfig {
    #[inline]
    fn eq(&self, other: &ProverConfig) -> bool {
        self.grpc_endpoint == other.grpc_endpoint && self.grpc == other.grpc
            && self.log == other.log && self.telemetry == other.telemetry
            && self.shutdown == other.shutdown
            && self.max_concurrency_limit == other.max_concurrency_limit
            && self.max_request_duration == other.max_request_duration
            && self.max_buffered_queries == other.max_buffered_queries
            && self.primary_prover == other.primary_prover
            && self.fallback_prover == other.fallback_prover
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ProverConfig {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<SocketAddr>;
        let _: ::core::cmp::AssertParamIsEq<GrpcConfig>;
        let _: ::core::cmp::AssertParamIsEq<Log>;
        let _: ::core::cmp::AssertParamIsEq<TelemetryConfig>;
        let _: ::core::cmp::AssertParamIsEq<ShutdownConfig>;
        let _: ::core::cmp::AssertParamIsEq<usize>;
        let _: ::core::cmp::AssertParamIsEq<Duration>;
        let _: ::core::cmp::AssertParamIsEq<ProverType>;
        let _: ::core::cmp::AssertParamIsEq<Option<ProverType>>;
    }
}
impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            grpc_endpoint: default_socket_addr(),
            log: Log::default(),
            telemetry: TelemetryConfig::default(),
            shutdown: ShutdownConfig::default(),
            max_concurrency_limit: default_max_concurrency_limit(),
            max_request_duration: default_max_request_duration(),
            max_buffered_queries: default_max_buffered_queries(),
            primary_prover: ProverType::NetworkProver(NetworkProverConfig::default()),
            fallback_prover: None,
            grpc: Default::default(),
        }
    }
}
impl ProverConfig {
    pub fn try_load(path: &Path) -> Result<Self, ConfigurationError> {
        let reader = std::fs::read_to_string(path)
            .map_err(|source| {
                ConfigurationError::UnableToReadConfigFile {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        let deserializer = toml::de::Deserializer::new(&reader);
        serde::Deserialize::deserialize(deserializer)
            .map_err(ConfigurationError::DeserializationError)
    }
}
#[serde(rename_all = "kebab-case")]
pub struct GrpcConfig {
    #[serde(
        skip_serializing_if = "same_as_default_max_decoding_message_size",
        default = "default_max_decoding_message_size"
    )]
    pub max_decoding_message_size: usize,
    #[serde(
        skip_serializing_if = "same_as_default_max_encoding_message_size",
        default = "default_max_encoding_message_size"
    )]
    pub max_encoding_message_size: usize,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for GrpcConfig {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "GrpcConfig",
                false as usize
                    + if same_as_default_max_decoding_message_size(
                        &self.max_decoding_message_size,
                    ) {
                        0
                    } else {
                        1
                    }
                    + if same_as_default_max_encoding_message_size(
                        &self.max_encoding_message_size,
                    ) {
                        0
                    } else {
                        1
                    },
            )?;
            if !same_as_default_max_decoding_message_size(
                &self.max_decoding_message_size,
            ) {
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "max-decoding-message-size",
                    &self.max_decoding_message_size,
                )?;
            } else {
                _serde::ser::SerializeStruct::skip_field(
                    &mut __serde_state,
                    "max-decoding-message-size",
                )?;
            }
            if !same_as_default_max_encoding_message_size(
                &self.max_encoding_message_size,
            ) {
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "max-encoding-message-size",
                    &self.max_encoding_message_size,
                )?;
            } else {
                _serde::ser::SerializeStruct::skip_field(
                    &mut __serde_state,
                    "max-encoding-message-size",
                )?;
            }
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for GrpcConfig {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "max-decoding-message-size" => {
                            _serde::__private::Ok(__Field::__field0)
                        }
                        "max-encoding-message-size" => {
                            _serde::__private::Ok(__Field::__field1)
                        }
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"max-decoding-message-size" => {
                            _serde::__private::Ok(__Field::__field0)
                        }
                        b"max-encoding-message-size" => {
                            _serde::__private::Ok(__Field::__field1)
                        }
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<GrpcConfig>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = GrpcConfig;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct GrpcConfig",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        usize,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_max_decoding_message_size(),
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        usize,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => default_max_encoding_message_size(),
                    };
                    _serde::__private::Ok(GrpcConfig {
                        max_decoding_message_size: __field0,
                        max_encoding_message_size: __field1,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<usize> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<usize> = _serde::__private::None;
                    while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max-decoding-message-size",
                                        ),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max-encoding-message-size",
                                        ),
                                    );
                                }
                                __field1 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => default_max_decoding_message_size(),
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => default_max_encoding_message_size(),
                    };
                    _serde::__private::Ok(GrpcConfig {
                        max_decoding_message_size: __field0,
                        max_encoding_message_size: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &[
                "max-decoding-message-size",
                "max-encoding-message-size",
            ];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "GrpcConfig",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<GrpcConfig>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for GrpcConfig {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "GrpcConfig",
            "max_decoding_message_size",
            &self.max_decoding_message_size,
            "max_encoding_message_size",
            &&self.max_encoding_message_size,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GrpcConfig {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GrpcConfig {
    #[inline]
    fn eq(&self, other: &GrpcConfig) -> bool {
        self.max_decoding_message_size == other.max_decoding_message_size
            && self.max_encoding_message_size == other.max_encoding_message_size
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for GrpcConfig {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<usize>;
    }
}
impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            max_decoding_message_size: default_max_decoding_message_size(),
            max_encoding_message_size: default_max_encoding_message_size(),
        }
    }
}
#[serde(rename_all = "kebab-case")]
pub struct ClientProverConfig {
    #[serde(default)]
    pub grpc: GrpcConfig,
}
#[automatically_derived]
impl ::core::default::Default for ClientProverConfig {
    #[inline]
    fn default() -> ClientProverConfig {
        ClientProverConfig {
            grpc: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ClientProverConfig {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "ClientProverConfig",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "grpc",
                &self.grpc,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for ClientProverConfig {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "grpc" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"grpc" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<ClientProverConfig>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = ClientProverConfig;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct ClientProverConfig",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        GrpcConfig,
                    >(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    _serde::__private::Ok(ClientProverConfig {
                        grpc: __field0,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<GrpcConfig> = _serde::__private::None;
                    while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("grpc"),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<GrpcConfig>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::Default::default(),
                    };
                    _serde::__private::Ok(ClientProverConfig {
                        grpc: __field0,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["grpc"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "ClientProverConfig",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<ClientProverConfig>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for ClientProverConfig {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "ClientProverConfig",
            "grpc",
            &&self.grpc,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ClientProverConfig {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ClientProverConfig {
    #[inline]
    fn eq(&self, other: &ClientProverConfig) -> bool {
        self.grpc == other.grpc
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ClientProverConfig {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<GrpcConfig>;
    }
}
const fn default_max_decoding_message_size() -> usize {
    4 * 1024 * 1024
}
fn same_as_default_max_decoding_message_size(value: &usize) -> bool {
    *value == default_max_decoding_message_size()
}
const fn default_max_encoding_message_size() -> usize {
    4 * 1024 * 1024
}
fn same_as_default_max_encoding_message_size(value: &usize) -> bool {
    *value == default_max_encoding_message_size()
}
const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}
const fn default_max_buffered_queries() -> usize {
    100
}
const fn default_max_request_duration() -> Duration {
    Duration::from_secs(60 * 5)
}
pub enum ConfigurationError {
    #[error("Unable to read the configuration file: {source}")]
    UnableToReadConfigFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to deserialize the configuration: {0}")]
    DeserializationError(#[from] toml::de::Error),
}
#[automatically_derived]
impl ::core::fmt::Debug for ConfigurationError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ConfigurationError::UnableToReadConfigFile {
                path: __self_0,
                source: __self_1,
            } => {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "UnableToReadConfigFile",
                    "path",
                    __self_0,
                    "source",
                    &__self_1,
                )
            }
            ConfigurationError::DeserializationError(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "DeserializationError",
                    &__self_0,
                )
            }
        }
    }
}
#[allow(unused_qualifications)]
#[automatically_derived]
impl ::thiserror::__private::Error for ConfigurationError {
    fn source(
        &self,
    ) -> ::core::option::Option<&(dyn ::thiserror::__private::Error + 'static)> {
        use ::thiserror::__private::AsDynError as _;
        #[allow(deprecated)]
        match self {
            ConfigurationError::UnableToReadConfigFile { source: source, .. } => {
                ::core::option::Option::Some(source.as_dyn_error())
            }
            ConfigurationError::DeserializationError { 0: source, .. } => {
                ::core::option::Option::Some(source.as_dyn_error())
            }
        }
    }
}
#[allow(unused_qualifications)]
#[automatically_derived]
impl ::core::fmt::Display for ConfigurationError {
    fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use ::thiserror::__private::AsDisplay as _;
        #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
        match self {
            ConfigurationError::UnableToReadConfigFile { path, source } => {
                match (source.as_display(),) {
                    (__display_source,) => {
                        __formatter
                            .write_fmt(
                                format_args!(
                                    "Unable to read the configuration file: {0}",
                                    __display_source,
                                ),
                            )
                    }
                }
            }
            ConfigurationError::DeserializationError(_0) => {
                match (_0.as_display(),) {
                    (__display0,) => {
                        __formatter
                            .write_fmt(
                                format_args!(
                                    "Failed to deserialize the configuration: {0}",
                                    __display0,
                                ),
                            )
                    }
                }
            }
        }
    }
}
#[allow(
    deprecated,
    unused_qualifications,
    clippy::elidable_lifetime_names,
    clippy::needless_lifetimes,
)]
#[automatically_derived]
impl ::core::convert::From<toml::de::Error> for ConfigurationError {
    fn from(source: toml::de::Error) -> Self {
        ConfigurationError::DeserializationError {
            0: source,
        }
    }
}
pub(crate) fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
